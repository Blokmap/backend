use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use location::Location;
use permissions::Permissions;

use crate::schemas::BuildResponse;
use crate::schemas::location::CreateLocationMemberRequest;
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
	Permissions::check_for_location(
		id,
		session.data.profile_id,
		Permissions::LocManageMembers
			| Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
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
	Permissions::check_for_location(
		id,
		session.data.profile_id,
		Permissions::LocManageMembers
			| Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
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
pub async fn delete_location_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((l_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_location(
		l_id,
		session.data.profile_id,
		Permissions::LocManageMembers
			| Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	Location::delete_member(l_id, p_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
