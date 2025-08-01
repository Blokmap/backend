use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use models::{OpeningTime, OpeningTimeIncludes, TimeBoundsFilter};

use crate::Session;
use crate::schemas::opening_time::{
	CreateOpeningTimeRequest,
	OpeningTimeResponse,
	UpdateOpeningTimeRequest,
};

#[instrument(skip(pool))]
pub async fn get_location_times(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
	Query(filter): Query<TimeBoundsFilter>,
	Query(includes): Query<OpeningTimeIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let times =
		OpeningTime::get_for_location(id, filter, includes, &conn).await?;
	let times: Vec<OpeningTimeResponse> =
		times.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(times)))
}

#[instrument(skip(pool))]
pub async fn create_location_time(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Query(includes): Query<OpeningTimeIncludes>,
	Json(request): Json<CreateOpeningTimeRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_time = request.to_insertable(id, session.data.profile_id);
	let new_time = new_time.insert(includes, &conn).await?;
	let response = OpeningTimeResponse::from(new_time);

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_location_time(
	State(pool): State<DbPool>,
	session: Session,
	Path((id, time_id)): Path<(i32, i32)>,
	Query(includes): Query<OpeningTimeIncludes>,
	Json(request): Json<UpdateOpeningTimeRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let time_update = request.to_insertable(session.data.profile_id);
	let updated_time = time_update.apply_to(time_id, includes, &conn).await?;
	let response = OpeningTimeResponse::from(updated_time);

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_location_time(
	State(pool): State<DbPool>,
	Path((id, time_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	OpeningTime::delete_by_id(time_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}
