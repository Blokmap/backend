use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use models::{Authority, AuthorityIncludes};

use crate::schemas::authority::AuthorityResponse;

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
pub async fn get_authority(
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authority = Authority::get_by_id(id, includes, &conn).await?;
	let _members = Authority::get_members(id, &conn).await?;

	Ok((StatusCode::OK, Json(authority)))
}
