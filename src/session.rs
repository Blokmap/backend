//! User sessions and tokens

use axum::RequestPartsExt;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::{Error, InternalServerError, RedisConn};
use models::PrimitiveProfile;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::AppState;

/// A session for any
///
/// ```rs
/// pub async fn foo_route(session: Session) -> impl IntoResponse {
///     println!("{:?}", session.data.profile_id);
///
///     ()
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Session {
	pub id:   i32,
	pub data: SessionData,
}

/// A session for any admin user
///
/// ```rs
/// pub async fn foo_route(session: AdminSession) -> impl IntoResponse {
///     println!("{:?}", session.data.profile_id);
///
///     ()
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AdminSession {
	pub id:   i32,
	pub data: SessionData,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SessionData {
	pub profile_id:       i32,
	pub profile_is_admin: bool,
}

impl FromRequestParts<AppState> for Session {
	type Rejection = Error;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let session_id = match parts.extensions.get::<i32>() {
			Some(id) => *id,
			None => {
				return Err(InternalServerError::SessionWithoutAuthError.into());
			},
		};

		let State(mut conn) = parts
			.extract_with_state::<State<RedisConn>, AppState>(state)
			.await
			.map_err(|_| Error::InternalServerError)?;

		let session = Self::get(session_id, &mut conn).await?;

		let Some(session) = session else {
			return Err(Error::Infallible(
				"failed to retrieve session despite passing auth middleware"
					.to_string(),
			));
		};

		Ok(session)
	}
}

impl FromRequestParts<AppState> for AdminSession {
	type Rejection = Error;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let session =
			parts.extract_with_state::<Session, AppState>(state).await?;

		if !session.data.profile_is_admin {
			return Err(Error::Forbidden);
		}

		let admin_session = Self { id: session.id, data: session.data };

		Ok(admin_session)
	}
}

impl Session {
	/// Create and store a new [`Session`] for a given [`Profile`]
	#[instrument(skip(conn))]
	pub async fn create(
		lifetime: Duration,
		profile: &PrimitiveProfile,
		conn: &mut RedisConn,
	) -> Result<Self, Error> {
		let id = profile.id;
		let profile_id = profile.id;

		let data =
			SessionData { profile_id, profile_is_admin: profile.is_admin };

		let session = Self { id, data };

		// Add a buffer of 10 seconds to ensure the cached session doesn't
		// expire before the session cookie does
		let expiry = lifetime.whole_seconds() + 10;

		let data = serde_json::to_string(&data)
			.map_err(InternalServerError::SerdeJsonError)?;

		let _: bool = conn.set(id, &data).await?;
		let _: bool = conn.expire(id, expiry).await?;

		debug!("stored session {} in cache for profile {}", id, profile.id);

		Ok(session)
	}

	/// Get a session from the cache
	#[instrument(skip(conn))]
	pub async fn get(
		id: i32,
		conn: &mut RedisConn,
	) -> Result<Option<Self>, Error> {
		let data_string: Option<String> = conn.get(id).await?;

		let Some(data_string) = data_string.as_ref() else {
			return Ok(None);
		};

		let data: SessionData = serde_json::from_str(data_string)
			.map_err(InternalServerError::SerdeJsonError)?;

		let session = Self { id, data };

		Ok(Some(session))
	}

	/// Remove a session given its id
	#[instrument(skip(conn))]
	pub async fn delete(id: i32, conn: &mut RedisConn) -> Result<(), Error> {
		let _: i32 = conn.del(id).await?;

		Ok(())
	}

	/// Check if a session with this id exists
	#[instrument(skip(conn))]
	pub async fn exists(id: i32, conn: &mut RedisConn) -> Result<bool, Error> {
		let exists: i32 = conn.exists(id).await?;

		Ok(exists == 1)
	}

	/// Convert this [`Session`] into an access token cookie
	#[must_use]
	pub fn to_access_token_cookie(
		self,
		name: String,
		lifetime: Duration,
		secure: bool,
	) -> Cookie<'static> {
		Cookie::build((name, self.id.to_string()))
			.http_only(true)
			.max_age(lifetime)
			.path("/")
			.same_site(SameSite::Lax)
			.secure(secure)
			.into()
	}
}
