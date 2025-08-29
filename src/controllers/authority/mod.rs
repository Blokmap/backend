use authority::{Authority, AuthorityIncludes, NewAuthorityMember};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use permissions::Permissions;

use crate::schemas::BuildResponse;
use crate::schemas::authority::{
	AuthorityResponse,
	CreateAuthorityRequest,
	UpdateAuthorityRequest,
};
use crate::{Config, Session};

mod location;
mod member;
mod role;

pub(crate) use location::*;
pub(crate) use member::*;
pub(crate) use role::*;

#[instrument(skip(pool))]
pub async fn create_authority(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<AuthorityIncludes>,
	Json(request): Json<CreateAuthorityRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_auth = request.to_insertable(session.data.profile_id);
	let auth = new_auth.insert(includes, &conn).await?;

	let new_member_req = NewAuthorityMember {
		authority_id: auth.primitive.id,
		profile_id:   session.data.profile_id,
		added_by:     session.data.profile_id,
	};
	new_member_req.insert(&conn).await?;

	let response = auth.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_all_authorities(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authorities = Authority::get_all(includes, &conn).await?;
	let response: Vec<AuthorityResponse> = authorities
		.into_iter()
		.map(|a| a.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_authority(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authority = Authority::get_by_id(id, includes, &conn).await?;
	let response = authority.build_response(includes, &config)?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_authority(
	State(config): State<Config>,
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
	let response = updated_auth.build_response(includes, &config)?;

	Ok((StatusCode::OK, Json(response)))
}
