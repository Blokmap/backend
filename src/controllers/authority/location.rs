use ::location::{Location, LocationIncludes};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use permissions::Permissions;

use crate::schemas::BuildResponse;
use crate::schemas::location::{CreateLocationRequest, LocationResponse};
use crate::{Config, Session};

#[instrument(skip(pool))]
pub(crate) async fn add_authority_location(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_authority(
		id,
		session.data.profile_id,
		Permissions::AuthAddLocations
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_location =
		request.to_insertable_for_authority(id, session.data.profile_id);
	let records = new_location.insert(includes, &conn).await?;
	let response = records.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn get_authority_locations(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Query(includes): Query<LocationIncludes>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let locations = Location::get_by_authority_id(id, includes, &conn).await?;
	let response: Vec<LocationResponse> = locations
		.into_iter()
		.map(|l| l.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}
