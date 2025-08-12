use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use models::{Institution, InstitutionCategory, InstitutionIncludes};

use crate::schemas::institution::InstitutionResponse;
use crate::schemas::pagination::PaginationOptions;

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
