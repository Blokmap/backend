use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use institution::Institution;
use permissions::Permissions;

use crate::schemas::BuildResponse;
use crate::schemas::institution::{
	CreateInstitutionMemberRequest,
	InstitutionMemberUpdateRequest,
};
use crate::schemas::profile::ProfileResponse;
use crate::{Config, Session};

#[instrument(skip(pool))]
pub(crate) async fn add_institution_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
	Json(request): Json<CreateInstitutionMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_institution(
		id,
		session.data.profile_id,
		Permissions::InstManageMembers | Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_inst_profile = request.to_insertable(id, session.data.profile_id);
	let member = new_inst_profile.insert(&conn).await?;
	let response = member.build_response((), &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_institution_members(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_institution(
		id,
		session.data.profile_id,
		Permissions::InstManageMembers | Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let members = Institution::get_members(id, &conn).await?;
	let response: Vec<ProfileResponse> = members
		.into_iter()
		.map(|data| data.build_response((), &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_insitution_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path((inst_id, prof_id)): Path<(i32, i32)>,
	Json(request): Json<InstitutionMemberUpdateRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_institution(
		inst_id,
		session.data.profile_id,
		Permissions::InstManageMembers | Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let member_update = request.to_insertable(session.data.profile_id);
	let updated_member =
		member_update.apply_to(inst_id, prof_id, &conn).await?;
	let response = updated_member.build_response((), &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_institution_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((i_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_institution(
		i_id,
		session.data.profile_id,
		Permissions::InstManageMembers | Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;
	Institution::delete_member(i_id, p_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}
