//! Controllers for authorization

use std::sync::LazyLock;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::{Path, State};
use axum::response::NoContent;
use axum::{Extension, Json};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use validator_derive::Validate;

use crate::models::{InsertableProfile, Profile, ProfileId};
use crate::{Config, DbPool, Error, TokenError};

static USERNAME_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9-_]{2,31}$").unwrap());

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct RegisterRequest {
	#[validate(regex(
		path = *USERNAME_REGEX,
		message = "username must start with a letter and only contain letters, numbers, dashes, or underscores",
		code = "username-regex"
	))]
	#[validate(length(
		min = 2,
		max = 32,
		message = "username must be between 2 and 32 characters long",
		code = "username-length"
	))]
	pub username: String,
	#[validate(length(
		min = 16,
		message = "password must be at least 16 characters long",
		code = "password-length"
	))]
	pub password: String,
	#[validate(email(message = "invalid email", code = "email"))]
	pub email:    String,
}

#[instrument(skip(pool, config))]
pub(crate) async fn register_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Json(register_data): Json<RegisterRequest>,
) -> Result<Json<Profile>, Error> {
	register_data.validate()?;

	let salt = SaltString::generate(&mut OsRng);
	let password_hash = Argon2::default()
		.hash_password(register_data.password.as_bytes(), &salt)?
		.to_string();

	let email_confirmation_token = Uuid::new_v4().to_string();
	let email_confirmation_token_expiry =
		Utc::now().naive_utc() + config.email_confirmation_token_lifetime;

	let insertable_profile = InsertableProfile {
		username: register_data.username,
		password_hash,
		pending_email: register_data.email,
		email_confirmation_token,
		email_confirmation_token_expiry,
	};

	let conn = pool.get().await?;
	let new_profile = insertable_profile.insert(conn).await?;

	// todo!("send confirmation email");

	info!(
		"registered new profile id: {} username: {} email: {}",
		new_profile.id,
		new_profile.username,
		new_profile.pending_email.clone().unwrap()
	);

	Ok(Json(new_profile))
}

#[instrument(skip(pool))]
pub(crate) async fn confirm_email(
	State(pool): State<DbPool>,
	Path(token): Path<String>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let profile = Profile::get_by_email_confirmation_token(token, conn).await?;

	// Unwrap is safe because profiles with a confirmation token will always
	// have a token expiry
	let expiry = profile.email_confirmation_token_expiry.unwrap();
	if Utc::now().naive_utc() < expiry {
		return Err(TokenError::ExpiredEmailToken.into());
	}

	let conn = pool.get().await?;
	profile.confirm_email(conn).await?;

	info!("confirmed email for profile {}", profile.id);

	Ok(NoContent)
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginUsernameRequest {
	username: String,
	password: String,
}

#[instrument(skip(pool, config))]
pub(crate) async fn login_profile_with_username(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Json(login_data): Json<LoginUsernameRequest>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let conn = pool.get().await?;
	let profile = Profile::get_by_username(login_data.username, conn).await?;

	let password_hash = PasswordHash::new(&profile.password_hash)?;
	Argon2::default()
		.verify_password(login_data.password.as_bytes(), &password_hash)?;

	let secure = config.production;
	let access_token =
		Cookie::build((config.access_token_name, profile.id.to_string()))
			.domain("")
			.http_only(true)
			.max_age(config.access_token_lifetime)
			.path("/")
			.same_site(SameSite::Lax)
			.secure(secure);

	let jar = jar.add(access_token);

	info!("logged in profile {} with username", profile.id);

	Ok((jar, NoContent))
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginEmailRequest {
	email:    String,
	password: String,
}

#[instrument(skip(pool, config))]
pub(crate) async fn login_profile_with_email(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Json(login_data): Json<LoginEmailRequest>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let conn = pool.get().await?;
	let profile = Profile::get_by_email(login_data.email, conn).await?;

	let password_hash = PasswordHash::new(&profile.password_hash)?;
	Argon2::default()
		.verify_password(login_data.password.as_bytes(), &password_hash)?;

	let secure = config.production;
	let access_token =
		Cookie::build((config.access_token_name, profile.id.to_string()))
			.domain("")
			.http_only(true)
			.max_age(config.access_token_lifetime)
			.path("/")
			.same_site(SameSite::Lax)
			.secure(secure);

	let jar = jar.add(access_token);

	info!("logged in profile {} with email", profile.id);

	Ok((jar, NoContent))
}

#[instrument(skip(config))]
pub(crate) async fn logout_profile(
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Extension(profile_id): Extension<ProfileId>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let secure = config.production;

	let revoked_access_token = Cookie::build((config.access_token_name, ""))
		.domain("")
		.http_only(true)
		.max_age(time::Duration::hours(-1))
		.path("/")
		.same_site(SameSite::Lax)
		.secure(secure);

	let jar = jar.add(revoked_access_token);

	info!("logged out profile {profile_id}");

	Ok((jar, NoContent))
}
