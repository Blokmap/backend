use ::authority::AuthorityIncludes;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use db::InstitutionCategory;
use institution::{Institution, InstitutionIncludes};

use crate::schemas::BuildResponse;
use crate::schemas::institution::{
	CreateInstitutionRequest,
	InstitutionResponse,
};
use crate::schemas::pagination::PaginationOptions;
use crate::{Config, Session};

mod authority;
mod member;

pub(crate) use authority::*;
pub(crate) use member::*;

#[instrument(skip(pool))]
pub async fn create_institution(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<InstitutionIncludes>,
	Json(request): Json<CreateInstitutionRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (new_institution, authority_request) =
		request.to_insertable(session.data.profile_id);
	let institution = new_institution.insert(includes, &conn).await?;
	let mut response = institution.build_response(includes, &config)?;

	if let Some(authority_request) = authority_request {
		let mut new_authority =
			authority_request.to_insertable(session.data.profile_id);
		new_authority.institution_id = Some(response.id);

		response.authority = Some(
			new_authority
				.insert(AuthorityIncludes::default(), &conn)
				.await?
				.build_response(AuthorityIncludes::default(), &config)?,
		);
	}

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_all_institutions(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	Query(includes): Query<InstitutionIncludes>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (total, truncated, institutions) =
		Institution::get_all(includes, p_opts.into(), &conn).await?;
	let institutions: Vec<InstitutionResponse> = institutions
		.into_iter()
		.map(|i| i.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	let response = p_opts.paginate(total, truncated, institutions);

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_institution(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	Query(includes): Query<InstitutionIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authority = Institution::get_by_id(id, includes, &conn).await?;
	let response = authority.build_response(includes, &config)?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument]
pub async fn get_categories() -> impl IntoResponse {
	(StatusCode::OK, Json(InstitutionCategory::get_variants()))
}
