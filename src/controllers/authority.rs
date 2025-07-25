use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use models::{
	Authority,
	AuthorityIncludes,
	AuthorityPermissions,
	Location,
	LocationIncludes,
	NewAuthorityProfile,
};

use crate::Session;
use crate::schemas::authority::{
	AuthorityResponse,
	CreateAuthorityMemberRequest,
	CreateAuthorityRequest,
	FullAuthorityResponse,
	UpdateAuthorityProfileRequest,
	UpdateAuthorityRequest,
};
use crate::schemas::location::{CreateLocationRequest, LocationResponse};
use crate::schemas::profile::ProfilePermissionsResponse;

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
		permissions:  AuthorityPermissions::Administrator.bits(),
	};
	new_member_req.insert(&conn).await?;

	let response: AuthorityResponse = auth.into();

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument]
pub async fn get_all_authority_permissions() -> impl IntoResponse {
	let perms = AuthorityPermissions::names();

	(StatusCode::OK, Json(perms))
}

#[instrument(skip(pool))]
pub async fn get_authority(
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authority = Authority::get_by_id(id, includes, &conn).await?;
	let members = Authority::get_members(id, &conn).await?;
	let locations = Location::get_simple_by_authority_id(
		id,
		LocationIncludes::default(),
		&conn,
	)
	.await?;

	let response = FullAuthorityResponse::from((authority, members, locations));

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
	let conn = pool.get().await?;

	let actor_id = session.data.profile_id;
	let actor_perms =
		Authority::get_member_permissions(id, actor_id, &conn).await?;

	if !actor_perms.intersects(
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageAuthority,
	) {
		return Err(Error::Forbidden);
	}

	let auth_update = request.to_insertable(actor_id);
	let updated_auth = auth_update.apply_to(id, includes, &conn).await?;
	let response: AuthorityResponse = updated_auth.into();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_authority_locations(
	State(pool): State<DbPool>,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let locations = Location::get_by_authority_id(id, includes, &conn).await?;
	let response: Vec<_> =
		locations.into_iter().map(LocationResponse::from).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn add_authority_location(
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let actor_id = session.data.profile_id;
	let actor_perms =
		Authority::get_member_permissions(id, actor_id, &conn).await?;

	if !actor_perms.intersects(
		AuthorityPermissions::Administrator | AuthorityPermissions::AddLocation,
	) {
		return Err(Error::Forbidden);
	}

	let new_location = request.to_insertable_for_authority(id, actor_id);
	let records = new_location.insert(includes, &conn).await?;
	let response = LocationResponse::from(records);

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_authority_members(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let members = Authority::get_members_with_permissions(id, &conn).await?;
	let response: Vec<_> =
		members.into_iter().map(ProfilePermissionsResponse::from).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn add_authority_member(
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<CreateAuthorityMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let actor_id = session.data.profile_id;
	let actor_perms =
		Authority::get_member_permissions(id, actor_id, &conn).await?;

	if !actor_perms.intersects(
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageMembers,
	) {
		return Err(Error::Forbidden);
	}

	let new_auth_profile = request.to_insertable(id, actor_id);
	let member = new_auth_profile.insert(&conn).await?;
	let response = ProfilePermissionsResponse::from(member);

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_authority_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((a_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let actor_id = session.data.profile_id;
	let actor_perms =
		Authority::get_member_permissions(a_id, actor_id, &conn).await?;

	if !actor_perms.intersects(
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageMembers,
	) {
		return Err(Error::Forbidden);
	}

	Authority::delete_member(a_id, p_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(pool))]
pub async fn update_authority_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((a_id, p_id)): Path<(i32, i32)>,
	Json(request): Json<UpdateAuthorityProfileRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let actor_id = session.data.profile_id;
	let actor_perms =
		Authority::get_member_permissions(a_id, actor_id, &conn).await?;

	if !actor_perms.intersects(
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageMembers,
	) {
		return Err(Error::Forbidden);
	}

	let auth_update = request.to_insertable(actor_id);
	let updated_member = auth_update.apply_to(a_id, p_id, &conn).await?;
	let response: ProfilePermissionsResponse = updated_member.into();

	Ok((StatusCode::OK, Json(response)))
}
