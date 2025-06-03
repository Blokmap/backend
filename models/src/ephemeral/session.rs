//! User sessions and tokens

use axum_extra::extract::cookie::{Cookie, SameSite};
use common::{Error, RedisConn};
use redis::AsyncCommands;
use time::Duration;
use uuid::Uuid;

use crate::Profile;

#[derive(Clone, Copy, Debug)]
pub struct Session {
	pub id:         Uuid,
	pub profile_id: i32,
}

impl Session {
	/// Create and store a new [`Session`] for a given [`Profile`]
	#[instrument(skip_all)]
	pub async fn create(
		lifetime: Duration,
		profile: &Profile,
		conn: &mut RedisConn,
	) -> Result<Self, Error> {
		let id = Uuid::new_v4();
		let profile_id = profile.id;

		let session = Self { id, profile_id };

		// Add a buffer of 10 seconds to ensure the cached session doesn't
		// expire before the session cookie does
		let expiry = lifetime.whole_seconds() + 10;

		let _: bool = conn.set(id, profile_id).await?;
		let _: bool = conn.expire(id, expiry).await?;

		debug!("stored session {} in cache for profile {}", id, profile.id);

		Ok(session)
	}

	/// Get a session from the cache
	///
	/// # Errors
	/// Fails if interacting with the cache fails
	pub async fn get(
		id: &Uuid,
		conn: &mut RedisConn,
	) -> Result<Option<Self>, Error> {
		let profile_id: Option<i32> = conn.get(id).await?;

		match profile_id {
			Some(profile_id) => Ok(Some(Self { id: *id, profile_id })),
			None => Ok(None),
		}
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

	/// Convert this [`Session`] into an refresh token cookie
	#[must_use]
	pub fn to_refresh_token_cookie(
		self,
		name: String,
		lifetime: Duration,
		secure: bool,
	) -> Cookie<'static> {
		Cookie::build((name, self.profile_id.to_string()))
			.http_only(true)
			.max_age(lifetime)
			.path("/")
			.same_site(SameSite::Lax)
			.secure(secure)
			.into()
	}
}
