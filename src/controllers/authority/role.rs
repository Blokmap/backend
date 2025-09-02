use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use permissions::{
	AuthorityPermissions,
	InstitutionPermissions,
	check_authority_perms,
};
use role::{AuthorityRole, RoleIncludes};

use crate::schemas::BuildResponse;
use crate::schemas::role::{
	CreateRoleRequest,
	RoleResponse,
	UpdateRoleRequest,
};
use crate::{Config, Session};

#[instrument(skip(pool))]
pub(crate) async fn create_authority_role(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path(auth_id): Path<i32>,
	Query(includes): Query<RoleIncludes>,
	Json(request): Json<CreateRoleRequest>,
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

	let new_role_req =
		request.to_insertable_for_authority(auth_id, session.data.profile_id);
	let new_role = new_role_req.insert(auth_id, includes, &conn).await?;
	let response = new_role.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn get_authority_roles(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path(auth_id): Path<i32>,
	Query(includes): Query<RoleIncludes>,
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

	let roles =
		AuthorityRole::get_for_authority(auth_id, includes, &conn).await?;
	let response: Vec<RoleResponse> = roles
		.into_iter()
		.map(|r| r.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn update_authority_role(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path((auth_id, role_id)): Path<(i32, i32)>,
	Query(includes): Query<RoleIncludes>,
	Json(request): Json<UpdateRoleRequest>,
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

	// Return not found if the role doesn't belong to this authority
	AuthorityRole::get_for_authority(auth_id, RoleIncludes::default(), &conn)
		.await?;

	let role_update =
		request.to_insertable_for_authority(session.data.profile_id);
	let updated_role = role_update.apply_to(role_id, includes, &conn).await?;
	let response = updated_role.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn delete_authority_role(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path((auth_id, role_id)): Path<(i32, i32)>,
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

	// Return not found if the role doesn't belong to this authority
	AuthorityRole::get_for_authority(auth_id, RoleIncludes::default(), &conn)
		.await?;

	AuthorityRole::delete_by_id(role_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
