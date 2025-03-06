//! Middleware to authorize users and store user data on the request objects

use std::pin::Pin;
use std::task::{Context, Poll};

use axum::RequestExt;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{Response, StatusCode};
use axum_extra::extract::PrivateCookieJar;
use tower::{Layer, Service};

use crate::AppState;
use crate::models::{Profile, ProfileId};

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

	#[instrument(skip(self))]
	fn call(&mut self, mut req: Request<Body>) -> Self::Future {
		let cloned_inner = self.inner.clone();
		let mut inner = std::mem::replace(&mut self.inner, cloned_inner);

		let state = self.state.clone();

		Box::pin(async move {
			let jar = req
				.extract_parts_with_state::<PrivateCookieJar, _>(&state)
				.await
				.unwrap();

			let Some(access_token) = jar.get(&state.config.access_token_name)
			else {
				info!("missing access token");

				return Ok(Response::builder()
					.status(StatusCode::FORBIDDEN)
					.body(().into())
					.unwrap());
			};

			let Ok(profile_id) = access_token.value().parse::<i32>() else {
				info!("invalid access token");

				return Ok(Response::builder()
					.status(StatusCode::FORBIDDEN)
					.body(().into())
					.unwrap());
			};

			let conn = match state.database_pool.get().await {
				Ok(conn) => conn,
				Err(e) => {
					error!("database error -- {e}");

					return Ok(Response::builder()
						.status(StatusCode::INTERNAL_SERVER_ERROR)
						.body(().into())
						.unwrap());
				},
			};

			if Profile::exists(profile_id, conn).await.is_ok_and(|x| x) {
				let profile_id = ProfileId(profile_id);
				req.extensions_mut().insert(profile_id);

				inner.call(req).await
			} else {
				info!("unknown profile");

				Ok(Response::builder()
					.status(StatusCode::FORBIDDEN)
					.body(().into())
					.unwrap())
			}
		})
	}
}
