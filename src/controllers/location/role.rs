use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use location::{LocationRole, LocationRoleIncludes};
use permissions::Permissions;

use crate::schemas::BuildResponse;
use crate::schemas::roles::{
	CreateLocationRoleRequest,
	RoleResponse,
	UpdateLocationRoleRequest,
};
use crate::{Config, Session};

#[instrument(skip(pool))]
pub(crate) async fn create_location_role(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path(loc_id): Path<i32>,
	Query(includes): Query<LocationRoleIncludes>,
	Json(request): Json<CreateLocationRoleRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_location(
		loc_id,
		session.data.profile_id,
		Permissions::LocManageMembers
			| Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_role_req = request.to_insertable(loc_id, session.data.profile_id);
	let new_role = new_role_req.insert(includes, &conn).await?;
	let response = new_role.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn get_location_roles(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path(loc_id): Path<i32>,
	Query(includes): Query<LocationRoleIncludes>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_location(
		loc_id,
		session.data.profile_id,
		Permissions::LocManageMembers
			| Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let roles = LocationRole::get_for_location(loc_id, includes, &conn).await?;
	let response: Vec<RoleResponse> = roles
		.into_iter()
		.map(|r| r.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn update_location_role(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path((loc_id, role_id)): Path<(i32, i32)>,
	Query(includes): Query<LocationRoleIncludes>,
	Json(request): Json<UpdateLocationRoleRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_location(
		loc_id,
		session.data.profile_id,
		Permissions::LocManageMembers
			| Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let role_update = request.to_insertable(session.data.profile_id);
	let updated_role = role_update.apply_to(role_id, includes, &conn).await?;
	let response = updated_role.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn delete_location_role(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path((loc_id, role_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_location(
		loc_id,
		session.data.profile_id,
		Permissions::LocManageMembers
			| Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;
	LocationRole::delete_by_id(role_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
