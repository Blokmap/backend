//! Controllers for [`Profile`]s

use axum::extract::State;
use axum::{Extension, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::mailer::Mailer;
use crate::models::{Profile, ProfileId, ProfileUpdate};
use crate::{Config, DbPool, Error};

#[instrument(skip(pool))]
pub(crate) async fn get_all_profiles(
	State(pool): State<DbPool>,
) -> Result<Json<Vec<Profile>>, Error> {
	let conn = pool.get().await?;
	let profiles = Profile::get_all(&conn).await?;

	Ok(Json(profiles))
}

#[instrument(skip(pool))]
pub(crate) async fn get_current_profile(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
) -> Result<Json<Profile>, Error> {
	let conn = pool.get().await?;
	let profile = Profile::get(*profile_id, &conn).await?;

	Ok(Json(profile))
}

#[instrument(skip(pool, config, mailer))]
pub(crate) async fn update_current_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	Extension(profile_id): Extension<ProfileId>,
	Json(update): Json<ProfileUpdate>,
) -> Result<Json<Profile>, Error> {
	let conn = pool.get().await?;

	let old_profile = Profile::get(*profile_id, &conn).await?;
	let mut updated_profile = update.apply_to(*profile_id, &conn).await?;

	info!("set new pending email for profile {}", updated_profile.id);

	if old_profile.pending_email != updated_profile.pending_email {
		let email_confirmation_token = Uuid::new_v4().to_string();
		let email_confirmation_token_expiry =
			Utc::now().naive_utc() + config.email_confirmation_token_lifetime;

		updated_profile.email_confirmation_token =
			Some(email_confirmation_token.clone());
		updated_profile.email_confirmation_token_expiry =
			Some(email_confirmation_token_expiry);

		updated_profile = updated_profile.update(&conn).await?;

		let confirmation_url = format!(
			"{}/confirm_email/{}",
			config.frontend_url, email_confirmation_token,
		);

		let mail = mailer.try_build_message(
			&updated_profile,
			"Confirm your email",
			&format!(
				"Please confirm your email by going to {confirmation_url}"
			),
		)?;

		mailer.send(mail).await?;

		info!(
			"sent email confirmation email for profile {}",
			updated_profile.id
		);
	}

	Ok(Json(updated_profile))
}
