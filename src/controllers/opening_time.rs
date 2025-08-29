use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use opening_time::{NewOpeningTime, OpeningTime, OpeningTimeIncludes};

use crate::schemas::BuildResponse;
use crate::schemas::opening_time::{
	CreateOpeningTimeRequest,
	OpeningTimeResponse,
	UpdateOpeningTimeRequest,
};
use crate::{Config, Session};

#[instrument(skip(pool))]
pub async fn create_location_opening_times(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Query(includes): Query<OpeningTimeIncludes>,
	Json(request): Json<Vec<CreateOpeningTimeRequest>>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_times: Vec<_> = request
		.into_iter()
		.map(|t| t.to_insertable(id, session.data.profile_id))
		.collect();
	let new_times =
		NewOpeningTime::bulk_insert(new_times, includes, &conn).await?;
	let response: Vec<OpeningTimeResponse> =
		new_times.into_iter().map(Into::into).collect();

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_location_opening_time(
	State(config): State<Config>,
	State(pool): State<DbPool>,
	session: Session,
	Path((id, time_id)): Path<(i32, i32)>,
	Query(includes): Query<OpeningTimeIncludes>,
	Json(request): Json<UpdateOpeningTimeRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let time_update = request.to_insertable(session.data.profile_id);
	let updated_time = time_update.apply_to(time_id, includes, &conn).await?;
	let response = updated_time.build_response(includes, &config)?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_location_opening_time(
	State(pool): State<DbPool>,
	Path((id, time_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	OpeningTime::delete_by_id(time_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}
