use authority::{Authority, AuthorityIncludes, NewAuthorityProfile};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use location::{Location, LocationIncludes};
use permissions::Permissions;

use crate::schemas::BuildResponse;
use crate::schemas::authority::{
	AuthorityResponse,
	CreateAuthorityMemberRequest,
	CreateAuthorityRequest,
	UpdateAuthorityRequest,
};
use crate::schemas::location::{CreateLocationRequest, LocationResponse};
use crate::schemas::profile::ProfileResponse;
use crate::{Config, Session};

#[instrument(skip(pool))]
pub async fn get_all_authorities(
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authorities = Authority::get_all(includes, &conn).await?;
	let response: Vec<AuthorityResponse> =
		authorities.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn create_authority(
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<AuthorityIncludes>,
	Json(request): Json<CreateAuthorityRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_auth = request.to_insertable(session.data.profile_id);
	let auth = new_auth.insert(includes, &conn).await?;

	let new_member_req = NewAuthorityProfile {
		authority_id: auth.authority.id,
		profile_id:   session.data.profile_id,
		added_by:     session.data.profile_id,
	};
	new_member_req.insert(&conn).await?;

	let response: AuthorityResponse = auth.into();

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_authority(
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authority = Authority::get_by_id(id, includes, &conn).await?;
	let response = AuthorityResponse::from(authority);

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_authority(
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<AuthorityIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<UpdateAuthorityRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_authority(
		id,
		session.data.profile_id,
		Permissions::AuthAdministrator | Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let auth_update = request.to_insertable(session.data.profile_id);
	let updated_auth = auth_update.apply_to(id, includes, &conn).await?;
	let response: AuthorityResponse = updated_auth.into();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_authority_locations(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let locations = Location::get_by_authority_id(id, includes, &conn).await?;
	let response: Result<Vec<LocationResponse>, Error> =
		locations.into_iter().map(|l| l.build_response(&config)).collect();
	let response = response?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn add_authority_location(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_authority(
		id,
		session.data.profile_id,
		Permissions::AuthAddLocations
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_location =
		request.to_insertable_for_authority(id, session.data.profile_id);
	let records = new_location.insert(includes, &conn).await?;
	let response: LocationResponse = records.build_response(&config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_authority_members(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_authority(
		id,
		session.data.profile_id,
		Permissions::AuthManageMembers
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let members = Authority::get_members(id, &conn).await?;
	let response: Vec<ProfileResponse> = members
		.into_iter()
		.map(|data| data.build_response(&config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn add_authority_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<CreateAuthorityMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_authority(
		id,
		session.data.profile_id,
		Permissions::AuthManageMembers
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_auth_profile = request.to_insertable(id, session.data.profile_id);
	let member = new_auth_profile.insert(&conn).await?;
	let response: ProfileResponse = member.build_response(&config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_authority_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((a_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_authority(
		a_id,
		session.data.profile_id,
		Permissions::AuthManageMembers
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;
	Authority::delete_member(a_id, p_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}
