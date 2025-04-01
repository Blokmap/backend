//! Middleware to authorize users and store user data on the request objects

use std::pin::Pin;
use std::task::{Context, Poll};

use axum::RequestExt;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum_extra::extract::PrivateCookieJar;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::models::ephemeral::Session;
use crate::models::{Profile, ProfileId};
use crate::{AppState, Error};

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
			let conn = match state.database_pool.get().await {
				Ok(conn) => conn,
				Err(e) => return Ok(Error::from(e).into_response()),
			};

			// Skip authorization in the development environment
			// with the Skip-Auth header in the request.
			if !state.config.production && req.headers().contains_key("Skip-Auth") {
				// In development, we can just query the first admin user.
				// We assume that the seeder has created an admin user.
				let profile = conn
					.interact(|conn| {
						use diesel::prelude::*;

						use crate::schema::profile::dsl::*;

						profile.filter(admin.eq(true)).first::<Profile>(conn)
					})
					.await
					.unwrap();

				let profile = match profile {
					Ok(p) => p,
					Err(e) => return Ok(Error::from(e).into_response()),
				};

				req.extensions_mut().insert(ProfileId(profile.id));

				info!("skipping auth in development mode");
				return inner.call(req).await;
			}

			let jar = req
				.extract_parts_with_state::<PrivateCookieJar, _>(&state)
				.await
				.unwrap();

			let mut r_conn = state.redis_connection;

			let Some(refresh_token) = jar.get(&state.config.refresh_token_name)
			else {
				info!("got request without valid refresh token");

				return Ok(Response::builder()
					.status(StatusCode::UNAUTHORIZED)
					.body(().into())
					.unwrap());
			};

			let Some(access_token) = jar.get(&state.config.access_token_name)
			else {
				// Unwrap is safe as correctly signed refresh tokens are always
				// i32
				let profile_id = refresh_token.value().parse::<i32>().unwrap();

				let exists = match Profile::exists(profile_id, &conn).await {
					Ok(b) => b,
					Err(e) => return Ok(e.into_response()),
				};

				if !exists {
					warn!(
						"attempted to create tokens for unknown profile {}",
						profile_id
					);

					return Ok(Response::builder()
						.status(StatusCode::FORBIDDEN)
						.body(().into())
						.unwrap());
				}

				let profile = match Profile::get(profile_id, &conn).await {
					Ok(p) => p,
					Err(e) => return Ok(e.into_response()),
				};

				let session =
					match Session::create(&state.config, &profile, &mut r_conn)
						.await
					{
						Ok(s) => s,
						Err(e) => return Ok(e.into_response()),
					};

				let access_token_cookie =
					session.to_access_token_cookie(&state.config);
				let refresh_token_cookie =
					session.to_refresh_token_cookie(&state.config);

				let jar =
					jar.add(access_token_cookie).add(refresh_token_cookie);

				let profile_id = ProfileId(session.profile_id);
				req.extensions_mut().insert(profile_id);

				return inner
					.call(req)
					.await
					.map(|res| (jar, res).into_response());
			};

			// Unwrap is safe as correctly signed access tokens are always Uuids
			let session_id = access_token.value().parse::<Uuid>().unwrap();

			let session = match Session::get(&session_id, &mut r_conn).await {
				Ok(s) => s,
				Err(e) => return Ok(e.into_response()),
			};

			let Some(session) = session else {
				warn!("attempted to authorize unknown session {}", session_id);

				return Ok(Response::builder()
					.status(StatusCode::FORBIDDEN)
					.body(().into())
					.unwrap());
			};

			let profile_id = ProfileId(session.profile_id);
			req.extensions_mut().insert(profile_id);

			inner.call(req).await
		})
	}
}
