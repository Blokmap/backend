use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use common::{DbPool, Error};
use models::{Tag, TagIncludes};

use crate::ProfileId;
use crate::schemas::tag::{CreateTagRequest, TagResponse, UpdateTagRequest};

#[instrument(skip(pool))]
pub async fn get_all_tags(
	State(pool): State<DbPool>,
	Query(includes): Query<TagIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let tags = Tag::get_all(includes, &conn).await?;
	let response: Vec<TagResponse> = tags.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn create_tag(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Query(includes): Query<TagIncludes>,
	Json(request): Json<CreateTagRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_tag = request.to_insertable(*profile_id);
	let tag = new_tag.insert(includes, &conn).await?;
	let response: TagResponse = tag.into();

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn update_tag(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Query(includes): Query<TagIncludes>,
	Path(id): Path<i32>,
	Json(request): Json<UpdateTagRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let tag_update = request.to_insertable(*profile_id);
	let updated_tag = tag_update.apply_to(id, includes, &conn).await?;
	let response: TagResponse = updated_tag.into();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_tag(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	Tag::delete_by_id(id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}
