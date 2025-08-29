//! Controllers for [`Profile`]s

use authority::{Authority, AuthorityIncludes};
use axum::extract::{Path, Query, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use axum::{Json, RequestExt};
use axum_extra::extract::PrivateCookieJar;
use common::{DbPool, Error, RedisConn};
use db::ProfileState;
use location::{Location, LocationIncludes};
use profile::{Profile, ProfileStats, UpdateProfile};
use reservation::{Reservation, ReservationFilter, ReservationIncludes};
use review::{Review, ReviewIncludes};
use uuid::Uuid;

use crate::mailer::Mailer;
use crate::schemas::BuildResponse;
use crate::schemas::authority::AuthorityResponse;
use crate::schemas::location::LocationResponse;
use crate::schemas::pagination::{PaginatedResponse, PaginationOptions};
use crate::schemas::profile::{
	ProfileResponse,
	ProfileStatsResponse,
	UpdateProfileRequest,
};
use crate::schemas::reservation::ReservationResponse;
use crate::schemas::review::ReviewResponse;
use crate::{AdminSession, AppState, Config, Session};

mod avatar;

pub(crate) use avatar::*;

/// Get all [`Profile`]s
#[instrument(skip(pool, config))]
pub async fn get_all_profiles(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<Json<PaginatedResponse<Vec<ProfileResponse>>>, Error> {
	let conn = pool.get().await?;

	let (total, truncated, profiles) =
		Profile::get_all(p_opts.into(), &conn).await?;

	let profiles: Vec<ProfileResponse> = profiles
		.into_iter()
		.map(|data| data.build_response((), &config))
		.collect::<Result<_, _>>()?;

	let paginated = p_opts.paginate(total, truncated, profiles);

	Ok(Json(paginated))
}

/// # Panics
/// Panics if the request doesn't have a valid cookie jar
#[instrument(skip(state, config, pool))]
pub async fn get_current_profile(
	State(state): State<AppState>,
	State(config): State<Config>,
	State(pool): State<DbPool>,
	mut req: Request,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let jar = req
		.extract_parts_with_state::<PrivateCookieJar, _>(&state)
		.await
		.unwrap();

	let mut r_conn = state.redis_connection;

	let Some(access_token) = jar.get(&state.config.access_token_name) else {
		return Ok((StatusCode::OK, Json(None)));
	};

	// Unwrap is safe as correctly signed access tokens are always i32
	let session_id = access_token.value().parse::<i32>().unwrap();

	let Ok(Some(session)) = Session::get(session_id, &mut r_conn).await else {
		return Ok((StatusCode::OK, Json(None)));
	};

	let profile = Profile::get(session.data.profile_id, &conn).await?;
	let response = profile.build_response((), &config)?;

	Ok((StatusCode::OK, Json(Some(response))))
}

#[instrument(skip(pool, config))]
pub async fn get_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	if !session.data.is_admin && p_id != session.data.profile_id {
		return Err(Error::Forbidden);
	}

	let profile = Profile::get(p_id, &conn).await?;
	let response = profile.build_response((), &config)?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool, config, mailer))]
pub async fn update_current_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	session: Session,
	Json(update): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let old_profile = Profile::get(session.data.profile_id, &conn).await?;

	let mut updated_profile = UpdateProfile::from(update)
		.apply_to(session.data.profile_id, &conn)
		.await?;

	if old_profile.primitive.pending_email
		!= updated_profile.primitive.pending_email
	{
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

		info!(
			"set new pending email for profile {}",
			updated_profile.primitive.id
		);
	}

	let response = updated_profile.build_response((), &config)?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool, config, mailer))]
pub async fn update_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	session: Session,
	Path(p_id): Path<i32>,
	Json(update): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	if !session.data.is_admin && p_id != session.data.profile_id {
		return Err(Error::Forbidden);
	}

	let old_profile = Profile::get(p_id, &conn).await?;

	let mut updated_profile =
		UpdateProfile::from(update).apply_to(p_id, &conn).await?;

	if old_profile.primitive.pending_email
		!= updated_profile.primitive.pending_email
	{
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

		info!(
			"set new pending email for profile {}",
			updated_profile.primitive.id
		);
	}

	let response = updated_profile.build_response((), &config)?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn disable_profile(
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	session: AdminSession,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let mut profile = Profile::get(profile_id, &conn).await?;

	profile.primitive.state = ProfileState::Disabled;
	profile.update(&conn).await?;

	Session::delete(profile_id, &mut r_conn).await?;

	info!("disabled profile {profile_id}");

	Ok(NoContent)
}

#[instrument(skip(pool))]
pub async fn activate_profile(
	State(pool): State<DbPool>,
	session: AdminSession,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let mut profile = Profile::get(profile_id, &conn).await?;

	profile.primitive.state = ProfileState::Active;
	profile.update(&conn).await?;

	info!("activated profile {profile_id}");

	Ok(NoContent)
}

#[instrument(skip(pool))]
pub async fn get_profile_locations(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Query(includes): Query<LocationIncludes>,
	Path(profile_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let locations =
		Location::get_by_profile_id(profile_id, includes, &conn).await?;
	let response: Vec<LocationResponse> = locations
		.into_iter()
		.map(|l| l.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_reservations(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	Query(filter): Query<ReservationFilter>,
	Query(includes): Query<ReservationIncludes>,
	Path(profile_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let reservations =
		Reservation::for_profile(profile_id, filter, includes, &conn).await?;
	let response: Vec<ReservationResponse> = reservations
		.into_iter()
		.map(|r| r.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_authorities(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authorities = Authority::for_profile(p_id, includes, &conn).await?;
	let response: Vec<AuthorityResponse> = authorities
		.into_iter()
		.map(|a| a.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_reviews(
	State(pool): State<DbPool>,
	Query(includes): Query<ReviewIncludes>,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let reviews = Review::for_profile(p_id, includes, &conn).await?;
	let response: Vec<ReviewResponse> =
		reviews.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_stats(
	State(pool): State<DbPool>,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let stats = ProfileStats::for_profile(p_id, &conn).await?;
	let response: ProfileStatsResponse = stats.into();

	Ok((StatusCode::OK, Json(response)))
}
