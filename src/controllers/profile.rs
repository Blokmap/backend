//! Controllers for [`Profile`]s

use std::sync::LazyLock;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use axum::Json;
use axum::extract::{Path, State};
use axum::response::NoContent;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use validator_derive::Validate;

use crate::error::{Error, TokenError};
use crate::models::{InsertableProfile, Profile};
use crate::{Config, DbPool};

static USERNAME_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9-_]{2,31}$").unwrap());

pub(crate) async fn get_all_profiles(
	State(pool): State<DbPool>,
) -> Result<Json<Vec<Profile>>, Error> {
	let conn = pool.get().await?;

	let profiles = Profile::get_all(conn).await?;

	Ok(Json(profiles))
}

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

pub(crate) async fn register_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Json(new_user): Json<RegisterRequest>,
) -> Result<Json<Profile>, Error> {
	new_user.validate()?;

	let salt = SaltString::generate(&mut OsRng);
	let argon2 = Argon2::default();
	let password_hash =
		argon2.hash_password(new_user.password.as_bytes(), &salt)?.to_string();

	let email_confirmation_token = Uuid::new_v4().to_string();
	let email_confirmation_token_expiry =
		Utc::now().naive_utc() + config.email_confirmation_token_lifetime;

	let insertable_profile = InsertableProfile {
		username: new_user.username,
		password_hash,
		pending_email: new_user.email,
		email_confirmation_token,
		email_confirmation_token_expiry,
	};

	let conn = pool.get().await?;
	let _new_profile = insertable_profile.insert(conn).await?;

	todo!("send confirmation email");

	// Ok(Json(new_profile))
}

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

	Ok(NoContent)
}
