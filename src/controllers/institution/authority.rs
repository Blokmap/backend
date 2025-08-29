use ::authority::{AuthorityIncludes, AuthorityUpdate};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use permissions::Permissions;

use crate::schemas::BuildResponse;
use crate::schemas::authority::CreateAuthorityRequest;
use crate::{Config, Session};

#[instrument(skip(pool))]
pub async fn create_institution_authority(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path(i_id): Path<i32>,
	Query(includes): Query<AuthorityIncludes>,
	Json(request): Json<CreateAuthorityRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_institution(
		i_id,
		session.data.profile_id,
		Permissions::InstAddAuthority | Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let mut new_authority = request.to_insertable(session.data.profile_id);
	new_authority.institution_id = Some(i_id);
	let new_authority = new_authority.insert(includes, &conn).await?;
	let response = new_authority.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn link_authority(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path((i_id, a_id)): Path<(i32, i32)>,
	Query(includes): Query<AuthorityIncludes>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_institution(
		i_id,
		session.data.profile_id,
		Permissions::InstAddAuthority | Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let update = AuthorityUpdate {
		name:           None,
		description:    None,
		updated_by:     session.data.profile_id,
		institution_id: Some(i_id),
	};
	let authority = update.apply_to(a_id, includes, &conn).await?;
	let response = authority.build_response(includes, &config)?;

	Ok((StatusCode::OK, Json(response)))
}
