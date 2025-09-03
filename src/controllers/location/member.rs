use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use location::Location;
use permissions::{
	AuthorityPermissions,
	InstitutionPermissions,
	LocationPermissions,
	check_location_perms,
};

use crate::schemas::BuildResponse;
use crate::schemas::location::{
	CreateLocationMemberRequest,
	LocationMemberUpdateRequest,
};
use crate::schemas::profile::ProfileResponse;
use crate::{Config, Session};

#[instrument(skip(pool))]
pub async fn add_location_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
	Json(request): Json<CreateLocationMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	check_location_perms(
		id,
		session.data.profile_id,
		LocationPermissions::ManageMembers | LocationPermissions::Administrator,
		AuthorityPermissions::Administrator,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_loc_profile = request.to_insertable(id, session.data.profile_id);
	let member = new_loc_profile.insert(&conn).await?;
	let response = member.build_response((), &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_location_members(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	check_location_perms(
		id,
		session.data.profile_id,
		LocationPermissions::ManageMembers | LocationPermissions::Administrator,
		AuthorityPermissions::Administrator,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let members = Location::get_members(id, &conn).await?;
	let response: Vec<ProfileResponse> = members
		.into_iter()
		.map(|data| data.build_response((), &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_location_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path((loc_id, prof_id)): Path<(i32, i32)>,
	Json(request): Json<LocationMemberUpdateRequest>,
) -> Result<impl IntoResponse, Error> {
	check_location_perms(
		loc_id,
		session.data.profile_id,
		LocationPermissions::ManageMembers | LocationPermissions::Administrator,
		AuthorityPermissions::Administrator,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let member_update = request.to_insertable(session.data.profile_id);
	let updated_member = member_update.apply_to(loc_id, prof_id, &conn).await?;
	let response = updated_member.build_response((), &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_location_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((l_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	check_location_perms(
		l_id,
		session.data.profile_id,
		LocationPermissions::ManageMembers | LocationPermissions::Administrator,
		AuthorityPermissions::Administrator,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	Location::delete_member(l_id, p_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
