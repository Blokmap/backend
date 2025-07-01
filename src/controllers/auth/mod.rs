//! Controllers for authorization

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;
use chrono::Utc;
use common::{DbPool, Error, LoginError, RedisConn, TokenError};
use models::{NewProfile, Profile, ProfileState};
use uuid::Uuid;
use validator::Validate;

use crate::mailer::Mailer;
use crate::schemas::auth::{
	LoginRequest,
	PasswordResetData,
	PasswordResetRequest,
	RegisterRequest,
};
use crate::{Config, Session};

pub mod sso;

#[instrument(skip_all)]
pub(crate) async fn register_profile(
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	jar: PrivateCookieJar,
	Json(register_data): Json<RegisterRequest>,
) -> Result<impl IntoResponse, Error> {
	register_data.validate()?;

	let email_confirmation_token = Uuid::new_v4().to_string();
	let email_confirmation_token_expiry =
		Utc::now().naive_utc() + config.email_confirmation_token_lifetime;

	let insertable_profile = NewProfile {
		username: register_data.username,
		password: register_data.password,
		pending_email: register_data.email,
		email_confirmation_token,
		email_confirmation_token_expiry,
	};

	let conn = pool.get().await?;
	let new_profile = insertable_profile.insert(&conn).await?;

	if !config.production && config.skip_verify {
		let profile = new_profile.confirm_email(&conn).await?;

		let session = Session::create(
			config.access_token_lifetime,
			&profile,
			&mut r_conn,
		)
		.await?;

		let access_token_cookie = session.to_access_token_cookie(
			config.access_token_name,
			config.access_token_lifetime,
			config.production,
		);

		let jar = jar.add(access_token_cookie);

		let profile = profile.update_last_login(&conn).await?;

		info!("confirmed email for profile {}", profile.id);

		Ok((StatusCode::CREATED, jar, Json(profile)).into_response())
	} else {
		// Unwrap is safe as the token was explicitly set in the insertable
		// profile
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

		Ok((StatusCode::CREATED, Json(new_profile)).into_response())
	}
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

	let session =
		Session::create(config.access_token_lifetime, &profile, &mut r_conn)
			.await?;

	let access_token_cookie = session.to_access_token_cookie(
		config.access_token_name,
		config.access_token_lifetime,
		config.production,
	);

	let jar = jar.add(access_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("confirmed email for profile {}", profile.id);

	Ok((jar, NoContent))
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

	let session =
		Session::create(config.access_token_lifetime, &profile, &mut r_conn)
			.await?;

	let access_token_cookie = session.to_access_token_cookie(
		config.access_token_name,
		config.access_token_lifetime,
		config.production,
	);

	let jar = jar.add(access_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("reset password for profile {}", profile.id);

	Ok((jar, NoContent))
}

#[instrument(skip_all)]
pub(crate) async fn login_profile(
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	State(config): State<Config>,
	jar: PrivateCookieJar,
	Json(login_data): Json<LoginRequest>,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let conn = pool.get().await?;
	let profile =
		Profile::get_by_email_or_username(login_data.username, &conn).await?;

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

	let session =
		Session::create(config.access_token_lifetime, &profile, &mut r_conn)
			.await?;

	let access_token_cookie = session.to_access_token_cookie(
		config.access_token_name,
		config.access_token_lifetime,
		config.production,
	);

	let jar = jar.add(access_token_cookie);

	let profile = profile.update_last_login(&conn).await?;

	info!("logged in profile {} with username", profile.id);

	Ok((jar, NoContent))
}

#[instrument(skip(config, jar))]
pub(crate) async fn logout_profile(
	State(config): State<Config>,
	State(mut r_conn): State<RedisConn>,
	jar: PrivateCookieJar,
	session: Session,
) -> Result<(PrivateCookieJar, NoContent), Error> {
	let access_token = Cookie::build(config.access_token_name).path("/");
	let jar = jar.remove(access_token);

	Session::delete(session.id, &mut r_conn).await?;

	info!("logged out profile {}", session.data.profile_id);

	Ok((jar, NoContent))
}
