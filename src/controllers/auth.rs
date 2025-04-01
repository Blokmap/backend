//! Controllers for authorization

use std::sync::LazyLock;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::NoContent;
use axum::{Extension, Json};
use axum_extra::extract::PrivateCookieJar;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use validator_derive::Validate;

use crate::mailer::Mailer;
use crate::models::ephemeral::Session;
use crate::models::{InsertableProfile, Profile, ProfileId, ProfileState};
use crate::{Config, DbPool, Error, LoginError, RedisConn, TokenError};

static USERNAME_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9-_]*$").unwrap());

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
		min = 8,
		message = "password must be at least 8 characters long",
		code = "password-length"
	))]
	pub password: String,
	#[validate(email(message = "invalid email", code = "email"))]
	pub email:    String,
}

#[instrument(skip_all)]
pub(crate) async fn register_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	Json(register_data): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Profile>), Error> {
	register_data.validate()?;

	let password_hash = Profile::hash_password(&register_data.password)?;

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
	let new_profile = insertable_profile.insert(&conn).await?;
	// Unwrap is safe as the token was explicitly set in the insertable profile
	let confirmation_token =
		new_profile.email_confirmation_token.clone().unwrap();

	mailer
		.send_confirm_email(
			&new_profile,
			&confirmation_token,
			&config.frontend_url,
		)
		.await?;

	info!(
		"registered new profile id: {} username: {} email: {}",
		new_profile.id,
		new_profile.username,
		new_profile.pending_email.clone().unwrap()
	);

	Ok((StatusCode::CREATED, Json(new_profile)))
}

pub(crate) async fn resend_confirmation_email(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let profile = Profile::get(profile_id, &conn).await?;

	let email_confirmation_token = Uuid::new_v4().to_string();

	let profile = profile
		.set_email_confirmation_token(
			&email_confirmation_token,
			config.email_confirmation_token_lifetime,
			&conn,
		)
		.await?;

	mailer
		.send_confirm_email(
			&profile,
			&email_confirmation_token,
			&config.frontend_url,
		)
		.await?;

	Ok(NoContent)
}

#[instrument(skip(pool, r_conn, config, jar))]
pub(crate) async fn confirm_email(
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Path(token): Path<String>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let conn = pool.get().await?;
	let profile =
		Profile::get_by_email_confirmation_token(token, &conn).await?;

	// Unwrap is safe because profiles with a confirmation token will always
	// have a token expiry
	let expiry = profile.email_confirmation_token_expiry.unwrap();
	if Utc::now().naive_utc() > expiry {
		return Err(TokenError::ExpiredEmailToken.into());
	}

	profile.confirm_email(&conn).await?;

	let session = Session::create(&config, &profile, &mut r_conn).await?;
	let access_token_cookie = session.to_access_token_cookie(&config);
	let refresh_token_cookie = session.to_refresh_token_cookie(&config);

	let jar = jar.add(access_token_cookie).add(refresh_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("confirmed email for profile {}", profile.id);

	Ok((jar, NoContent))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PasswordResetRequest {
	pub username: String,
}

#[instrument(skip(pool, config, mailer, request))]
pub(crate) async fn request_password_reset(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	Json(request): Json<PasswordResetRequest>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let profile = Profile::get_by_username(request.username, &conn).await?;

	let password_reset_token = Uuid::new_v4().to_string();

	let profile = profile
		.set_password_reset_token(
			&password_reset_token,
			config.password_reset_token_lifetime,
			&conn,
		)
		.await?;

	mailer
		.send_reset_password(
			&profile,
			&password_reset_token,
			&config.frontend_url,
		)
		.await?;

	Ok(NoContent)
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct PasswordResetData {
	pub token:    String,
	#[validate(length(
		min = 16,
		message = "password must be at least 16 characters long",
		code = "password-length"
	))]
	pub password: String,
}

#[instrument(skip_all)]
pub(crate) async fn reset_password(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mut r_conn): State<RedisConn>,
	jar: PrivateCookieJar,
	Json(request): Json<PasswordResetData>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	request.validate()?;

	let conn = pool.get().await?;
	let profile =
		Profile::get_by_password_reset_token(request.token, &conn).await?;

	// Unwrap is safe because profiles with a reset token will always
	// have a token expiry
	let expiry = profile.password_reset_token_expiry.unwrap();
	if Utc::now().naive_utc() > expiry {
		return Err(TokenError::ExpiredPasswordToken.into());
	}

	let profile = profile.change_password(&request.password, &conn).await?;

	let session = Session::create(&config, &profile, &mut r_conn).await?;
	let access_token_cookie = session.to_access_token_cookie(&config);
	let refresh_token_cookie = session.to_refresh_token_cookie(&config);

	let jar = jar.add(access_token_cookie).add(refresh_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("reset password for profile {}", profile.id);

	Ok((jar, NoContent))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginUsernameRequest {
	pub username: String,
	pub password: String,
}

#[instrument(skip_all)]
pub(crate) async fn login_profile_with_username(
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Json(login_data): Json<LoginUsernameRequest>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let conn = pool.get().await?;
	let profile = Profile::get_by_username(login_data.username, &conn).await?;

	match profile.state {
		ProfileState::Active => (),
		ProfileState::Disabled => return Err(LoginError::Disabled.into()),
		ProfileState::PendingEmailVerification => {
			return Err(LoginError::PendingEmailVerification.into());
		},
	}

	let password_hash = PasswordHash::new(&profile.password_hash)?;
	Argon2::default()
		.verify_password(login_data.password.as_bytes(), &password_hash)?;

	let session = Session::create(&config, &profile, &mut r_conn).await?;
	let access_token_cookie = session.to_access_token_cookie(&config);
	let refresh_token_cookie = session.to_refresh_token_cookie(&config);

	let jar = jar.add(access_token_cookie).add(refresh_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("logged in profile {} with username", profile.id);

	Ok((jar, NoContent))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginEmailRequest {
	pub email:    String,
	pub password: String,
}

#[instrument(skip_all)]
pub(crate) async fn login_profile_with_email(
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Json(login_data): Json<LoginEmailRequest>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let conn = pool.get().await?;
	let profile = Profile::get_by_email(login_data.email, &conn).await?;

	match profile.state {
		ProfileState::Active => (),
		ProfileState::Disabled => return Err(LoginError::Disabled.into()),
		ProfileState::PendingEmailVerification => {
			return Err(LoginError::PendingEmailVerification.into());
		},
	}

	let password_hash = PasswordHash::new(&profile.password_hash)?;
	Argon2::default()
		.verify_password(login_data.password.as_bytes(), &password_hash)?;

	let session = Session::create(&config, &profile, &mut r_conn).await?;
	let access_token_cookie = session.to_access_token_cookie(&config);
	let refresh_token_cookie = session.to_refresh_token_cookie(&config);

	let jar = jar.add(access_token_cookie).add(refresh_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("logged in profile {} with email", profile.id);

	Ok((jar, NoContent))
}

#[instrument(skip(config, jar))]
pub(crate) async fn logout_profile(
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Extension(profile_id): Extension<ProfileId>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	// Unwrap is safe because the auth middleware guarantees the token exists
	let mut revoked_access_token = jar.get(&config.access_token_name).unwrap();
	revoked_access_token.make_removal();

	let jar = jar.add(revoked_access_token);

	info!("logged out profile {profile_id}");

	Ok((jar, NoContent))
}
