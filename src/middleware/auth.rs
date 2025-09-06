//! Middleware to authorize users and store user data on the request objects

use std::pin::Pin;
use std::task::{Context, Poll};

use axum::RequestExt;
use axum::body::Body;
use axum::extract::Request;
use axum::http::Response;
use axum::response::IntoResponse;
use axum_extra::extract::PrivateCookieJar;
use common::{Error, TokenError};
use profile::{Profile, ProfileClaims};
use tower::{Layer, Service};

use crate::AppState;
use crate::session::Session;

/// Middleware layer that guarantees a request has a valid access token and
/// associated session
///
/// If a valid session is found its ID is stored as an
/// [`Extension`](axum::Extension)
///
/// This function does not extract any session data, controllers that need this
/// data should ask for a [`Session`] in their arguments
#[derive(Clone)]
pub struct AuthLayer {
	state: AppState,
}

impl AuthLayer {
	#[must_use]
	pub fn new(state: AppState) -> Self { Self { state } }
}

impl<S> Layer<S> for AuthLayer {
	type Service = AuthMiddleware<S>;

	fn layer(&self, inner: S) -> Self::Service {
		AuthMiddleware { inner, state: self.state.clone() }
	}
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
	inner: S,
	state: AppState,
}

impl<S> Service<Request<Body>> for AuthMiddleware<S>
where
	S: Service<Request, Response = Response<Body>> + Clone + Send + 'static,
	S::Future: Send + 'static,
{
	type Error = S::Error;
	type Future = Pin<
		Box<
			dyn Future<Output = Result<Self::Response, Self::Error>>
				+ Send
				+ 'static,
		>,
	>;
	type Response = S::Response;

	fn poll_ready(
		&mut self,
		cx: &mut Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	#[instrument(skip_all)]
	fn call(&mut self, mut req: Request<Body>) -> Self::Future {
		let cloned_inner = self.inner.clone();
		let mut inner = std::mem::replace(&mut self.inner, cloned_inner);

		let state = self.state.clone();

		Box::pin(async move {
			let mut jar = req
				.extract_parts_with_state::<PrivateCookieJar, _>(&state)
				.await
				.unwrap();

			let mut r_conn = state.redis_connection;
			let pool = state.database_pool;
			let conn = match pool.get().await {
				Ok(c) => c,
				Err(e) => {
					return Ok(Error::from(e).into_response());
				},
			};

			if let Some(claims_cookie) =
				jar.get(&state.config.claims_cookie_name)
			{
				let claims = match serde_json::from_str::<ProfileClaims>(
					claims_cookie.value(),
				) {
					Ok(c) => c,
					Err(e) => {
						return Ok(Error::from(e).into_response());
					},
				};

				let profile = match Profile::from_claims(claims, &conn).await {
					Ok(p) => p,
					Err(e) => {
						return Ok(e.into_response());
					},
				};

				let session = Session::create(
					state.config.access_cookie_lifetime,
					&profile,
					&mut r_conn,
				)
				.await;

				let session = match session {
					Ok(s) => s,
					Err(e) => {
						return Ok(e.into_response());
					},
				};

				let access_token_cookie = session.to_access_token_cookie(
					state.config.access_cookie_name.clone(),
					state.config.access_cookie_lifetime,
					state.config.production,
				);

				jar = jar.add(access_token_cookie);
			}

			let Some(access_token) = jar.get(&state.config.access_cookie_name)
			else {
				info!("got request without valid access token");

				return Ok(
					Error::from(TokenError::MissingAccessToken).into_response()
				);
			};

			// Unwrap is safe as correctly signed access tokens are always i32
			let session_id = access_token.value().parse::<i32>().unwrap();

			let exists = match Session::exists(session_id, &mut r_conn).await {
				Ok(s) => s,
				Err(e) => return Ok(e.into_response()),
			};

			if !exists {
				warn!("attempted to authorize unknown session {}", session_id);

				return Ok(
					Error::from(TokenError::MissingSession).into_response()
				);
			}

			req.extensions_mut().insert(session_id);

			let res = inner.call(req).await;

			res.map(|r| {
				let (head, body) = r.into_parts();

				let mut res = (jar, body).into_response();

				*res.status_mut() = head.status;
				*res.version_mut() = head.version;

				res.headers_mut().extend(head.headers);
				res.extensions_mut().extend(head.extensions);

				res
			})
		})
	}
}
