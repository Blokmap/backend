use std::pin::Pin;

use axum::body::Body;
use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use tower::{Layer, Service};

use crate::models::{Profile, ProfileId};
use crate::{AppState, Error};

#[derive(Clone)]
pub struct AdminLayer {
	state: AppState,
}

impl AdminLayer {
	#[must_use]
	pub fn new(state: AppState) -> Self { Self { state } }
}

impl<S> Layer<S> for AdminLayer {
	type Service = AdminMiddleware<S>;

	fn layer(&self, inner: S) -> Self::Service {
		AdminMiddleware { inner, state: self.state.clone() }
	}
}

#[derive(Clone)]
pub struct AdminMiddleware<S> {
	inner: S,
	state: AppState,
}

impl<S> Service<Request<Body>> for AdminMiddleware<S>
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
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}

	#[instrument(skip_all)]
	fn call(&mut self, req: Request<Body>) -> Self::Future {
		let cloned_inner = self.inner.clone();
		let mut inner = std::mem::replace(&mut self.inner, cloned_inner);

		let state = self.state.clone();

		Box::pin(async move {
			let Some(&profile_id) = req.extensions().get::<ProfileId>() else {
                debug!("Profile ID not found in request extensions");
				return Ok(Error::Forbidden.into_response());
			};

			let conn = match state.database_pool.get().await {
				Ok(conn) => conn,
				Err(e) => return Ok(Error::from(e).into_response()),
			};

			let profile = match Profile::get(*profile_id, &conn).await {
				Ok(p) => p,
				Err(e) => return Ok(e.into_response()),
			};

			if !profile.admin {
                debug!("Profile ID {} is not an admin", profile_id);
				return Ok(Error::Forbidden.into_response());
			}

			inner.call(req).await
		})
	}
}
