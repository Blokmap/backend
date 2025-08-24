use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use db::InstitutionCategory;
use models::{
	AuthorityIncludes,
	AuthorityUpdate,
	Institution,
	InstitutionIncludes,
	InstitutionPermissions,
};

use crate::schemas::BuildResponse;
use crate::schemas::authority::{AuthorityResponse, CreateAuthorityRequest};
use crate::schemas::institution::{
	CreateInstitutionMemberRequest,
	CreateInstitutionRequest,
	InstitutionResponse,
	UpdateInstitutionProfileRequest,
};
use crate::schemas::pagination::PaginationOptions;
use crate::schemas::profile::ProfilePermissionsResponse;
use crate::{Config, Session};

#[instrument(skip(pool))]
pub async fn create_institution(
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<InstitutionIncludes>,
	Json(request): Json<CreateInstitutionRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: permissions

	let (new_institution, authority_request) =
		request.to_insertable(session.data.profile_id);
	let institution = new_institution.insert(includes, &conn).await?;
	let mut response: InstitutionResponse = institution.into();

	if let Some(authority_request) = authority_request {
		let mut new_authority =
			authority_request.to_insertable(session.data.profile_id);
		new_authority.institution_id = Some(response.id);

		response.authority = Some(
			new_authority
				.insert(AuthorityIncludes::default(), &conn)
				.await?
				.into(),
		);
	}

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn create_institution_authority(
	State(pool): State<DbPool>,
	session: Session,
	Path(i_id): Path<i32>,
	Query(includes): Query<AuthorityIncludes>,
	Json(request): Json<CreateAuthorityRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: permissions

	let mut new_authority = request.to_insertable(session.data.profile_id);
	new_authority.institution_id = Some(i_id);
	let new_authority = new_authority.insert(includes, &conn).await?;
	let response: AuthorityResponse = new_authority.into();

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn link_authority(
	State(pool): State<DbPool>,
	session: Session,
	Path((i_id, a_id)): Path<(i32, i32)>,
	Query(includes): Query<AuthorityIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: permissions

	let update = AuthorityUpdate {
		name:           None,
		description:    None,
		updated_by:     session.data.profile_id,
		institution_id: Some(i_id),
	};
	let authority = update.apply_to(a_id, includes, &conn).await?;
	let response: AuthorityResponse = authority.into();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_all_institutions(
	State(pool): State<DbPool>,
	Query(includes): Query<InstitutionIncludes>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (total, truncated, institutions) =
		Institution::get_all(includes, p_opts.limit(), p_opts.offset(), &conn)
			.await?;
	let institutions: Vec<InstitutionResponse> =
		institutions.into_iter().map(Into::into).collect();

	let response = p_opts.paginate(total, truncated, institutions);

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_institution(
	State(pool): State<DbPool>,
	Query(includes): Query<InstitutionIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authority = Institution::get_by_id(id, includes, &conn).await?;
	let response = InstitutionResponse::from(authority);

	Ok((StatusCode::OK, Json(response)))
}

#[instrument]
pub async fn get_categories() -> impl IntoResponse {
	(StatusCode::OK, Json(InstitutionCategory::get_variants()))
}

#[instrument]
pub async fn get_all_institution_permissions() -> impl IntoResponse {
	let perms = InstitutionPermissions::names();

	(StatusCode::OK, Json(perms))
}

#[instrument(skip(pool))]
pub async fn get_institution_members(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let members = Institution::get_members_with_permissions(id, &conn).await?;
	let response: Vec<ProfilePermissionsResponse> = members
		.into_iter()
		.map(|data| data.build_response(&config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn add_institution_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
	Json(request): Json<CreateInstitutionMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: better permissions

	let actor_id = session.data.profile_id;
	let actor_perms =
		Institution::get_member_permissions(id, actor_id, &conn).await?;

	if !actor_perms.intersects(InstitutionPermissions::Administrator) {
		return Err(Error::Forbidden);
	}

	let new_inst_profile = request.to_insertable(id, actor_id);
	let (member, img, perms) = new_inst_profile.insert(&conn).await?;
	let response: ProfilePermissionsResponse =
		(member, img, perms).build_response(&config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_institution_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((i_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: better permissions

	let actor_id = session.data.profile_id;
	let actor_perms =
		Institution::get_member_permissions(i_id, actor_id, &conn).await?;

	if !actor_perms.intersects(InstitutionPermissions::Administrator) {
		return Err(Error::Forbidden);
	}

	Institution::delete_member(i_id, p_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}

#[instrument(skip(pool))]
pub async fn update_institution_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path((i_id, p_id)): Path<(i32, i32)>,
	Json(request): Json<UpdateInstitutionProfileRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: better permissions

	let actor_id = session.data.profile_id;
	let actor_perms =
		Institution::get_member_permissions(i_id, actor_id, &conn).await?;

	if !actor_perms.intersects(InstitutionPermissions::Administrator) {
		return Err(Error::Forbidden);
	}

	let inst_update = request.to_insertable(actor_id);
	let (updated_member, img, perms) =
		inst_update.apply_to(i_id, p_id, &conn).await?;
	let response: ProfilePermissionsResponse =
		(updated_member, img, perms).build_response(&config)?;

	Ok((StatusCode::OK, Json(response)))
}
