use authority::Authority;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use location::LocationIncludes;
use permissions::{
	AuthorityPermissions,
	InstitutionPermissions,
	check_authority_perms,
};

use crate::schemas::BuildResponse;
use crate::schemas::authority::{
	AuthorityMemberUpdateRequest,
	CreateAuthorityMemberRequest,
};
use crate::schemas::profile::ProfileResponse;
use crate::{Config, Session};

#[instrument(skip(pool))]
pub(crate) async fn add_authority_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<CreateAuthorityMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	check_authority_perms(
		id,
		session.data.profile_id,
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageMembers,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_auth_profile = request.to_insertable(id, session.data.profile_id);
	let member = new_auth_profile.insert(&conn).await?;
	let response = member.build_response((), &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn get_authority_members(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	check_authority_perms(
		id,
		session.data.profile_id,
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageMembers,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let members = Authority::get_members(id, &conn).await?;
	let response: Vec<ProfileResponse> = members
		.into_iter()
		.map(|p| p.build_response((), &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_authority_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path((auth_id, prof_id)): Path<(i32, i32)>,
	Json(request): Json<AuthorityMemberUpdateRequest>,
) -> Result<impl IntoResponse, Error> {
	check_authority_perms(
		auth_id,
		session.data.profile_id,
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageMembers,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let member_update = request.to_insertable(session.data.profile_id);
	let updated_member =
		member_update.apply_to(auth_id, prof_id, &conn).await?;
	let response = updated_member.build_response((), &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn delete_authority_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((a_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	check_authority_perms(
		a_id,
		session.data.profile_id,
		AuthorityPermissions::Administrator
			| AuthorityPermissions::ManageMembers,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;
	Authority::delete_member(a_id, p_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}
