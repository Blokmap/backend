//! Controllers for [`Profile`]s

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use models::{
	Authority,
	AuthorityIncludes,
	Location,
	LocationIncludes,
	Profile,
	ProfileState,
	Reservation,
	ReservationIncludes,
	UpdateProfile,
};
use uuid::Uuid;

use crate::mailer::Mailer;
use crate::schemas::authority::AuthorityResponse;
use crate::schemas::location::LocationResponse;
use crate::schemas::pagination::{PaginationOptions, PaginationResponse};
use crate::schemas::profile::{ProfileResponse, UpdateProfileRequest};
use crate::schemas::reservation::ReservationResponse;
use crate::{AdminSession, Config, Session};

/// Get all [`Profile`]s
#[instrument(skip(pool))]
pub(crate) async fn get_all_profiles(
	State(pool): State<DbPool>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<Json<PaginationResponse<Vec<ProfileResponse>>>, Error> {
	let conn = pool.get().await?;

	let (total, profiles) =
		Profile::get_all(p_opts.limit(), p_opts.offset(), &conn).await?;

	let profiles: Vec<ProfileResponse> =
		profiles.into_iter().map(Into::into).collect();

	let paginated = p_opts.paginate(total, profiles);

	Ok(Json(paginated))
}

/// Get a [`Profile`] given its id
#[instrument(skip(pool))]
pub(crate) async fn get_current_profile(
	State(pool): State<DbPool>,
	session: Session,
) -> Result<Json<ProfileResponse>, Error> {
	let conn = pool.get().await?;

	let profile = Profile::get(session.data.profile_id, &conn).await?;

	Ok(Json(profile.into()))
}

#[instrument(skip(pool, config, mailer))]
pub(crate) async fn update_current_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	session: Session,
	Json(update): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, Error> {
	let conn = pool.get().await?;

	let old_profile = Profile::get(session.data.profile_id, &conn).await?;

	let mut updated_profile = UpdateProfile::from(update)
		.apply_to(session.data.profile_id, &conn)
		.await?;

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

	Ok(Json(updated_profile.into()))
}

#[instrument(skip(pool))]
pub(crate) async fn disable_profile(
	State(pool): State<DbPool>,
	session: AdminSession,
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
	session: AdminSession,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let mut profile = Profile::get(profile_id, &conn).await?;

	profile.state = ProfileState::Active;
	profile.update(&conn).await?;

	info!("disabled profile {profile_id}");

	Ok(NoContent)
}

#[instrument(skip(pool))]
pub(crate) async fn get_profile_locations(
	State(pool): State<DbPool>,
	Query(includes): Query<LocationIncludes>,
	Path(profile_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;
	let locations =
		Location::get_by_profile_id(profile_id, includes, &conn).await?;
	let response: Vec<LocationResponse> =
		locations.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_reservations(
	State(pool): State<DbPool>,
	Query(includes): Query<ReservationIncludes>,
	Path(profile_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let reservations =
		Reservation::for_profile(profile_id, includes, &conn).await?;
	let response: Vec<ReservationResponse> =
		reservations.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_authorities(
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authorities = Authority::for_profile(p_id, includes, &conn).await?;
	let response: Vec<AuthorityResponse> =
		authorities.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}
