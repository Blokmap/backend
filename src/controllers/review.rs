use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use common::{DbPool, Error};
use models::Review;

use crate::Session;
use crate::schemas::review::{
	CreateReviewRequest,
	ReviewResponse,
	UpdateReviewRequest,
};

#[instrument(skip(pool))]
pub async fn get_location_reviews(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let reviews = Review::for_location(id, &conn).await?;
	let response: Vec<_> =
		reviews.into_iter().map(ReviewResponse::from).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn create_location_review(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Json(request): Json<CreateReviewRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_review = request.to_insertable(session.data.profile_id, id)?;
	let review = new_review.insert(&conn).await?;
	let response: ReviewResponse = review.into();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_location_review(
	State(pool): State<DbPool>,
	Path((l_id, r_id)): Path<(i32, i32)>,
	Json(request): Json<UpdateReviewRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let review_update = request.to_insertable()?;
	let updated_review = review_update.apply_to(r_id, &conn).await?;
	let response: ReviewResponse = updated_review.into();

	Ok((StatusCode::OK, Json(response)))
}
