//! Controllers for [`Profile`]s

use axum::extract::{Path, State};
use axum::response::NoContent;
use axum::{Extension, Json};
use uuid::Uuid;

use crate::mailer::Mailer;
use crate::models::{Profile, ProfileId, ProfileState, ProfileUpdate};
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

	if old_profile.pending_email != updated_profile.pending_email {
		let email_confirmation_token = Uuid::new_v4().to_string();

		updated_profile = updated_profile
			.set_email_confirmation_token(
				&email_confirmation_token,
				config.email_confirmation_token_lifetime,
				&conn,
			)
			.await?;

		mailer
			.send_confirm_email(
				&updated_profile,
				&email_confirmation_token,
				&config.frontend_url,
			)
			.await?;

		info!("set new pending email for profile {}", updated_profile.id);
	}

	Ok(Json(updated_profile))
}

#[instrument(skip(pool))]
pub(crate) async fn disable_profile(
	State(pool): State<DbPool>,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let mut profile = Profile::get(profile_id, &conn).await?;

	profile.state = ProfileState::Disabled;
	profile.update(&conn).await?;

	info!("disabled profile {profile_id}");

	Ok(NoContent)
}

#[instrument(skip(pool))]
pub(crate) async fn activate_profile(
	State(pool): State<DbPool>,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let mut profile = Profile::get(profile_id, &conn).await?;

	profile.state = ProfileState::Active;
	profile.update(&conn).await?;

	info!("disabled profile {profile_id}");

	Ok(NoContent)
}
