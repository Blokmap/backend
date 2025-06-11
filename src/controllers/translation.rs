//! Controllers for [`Translation`]s

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use models::{Translation, TranslationIncludes};

use crate::Session;
use crate::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
	UpdateTranslationRequest,
};

/// Create and store a single translation in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_translation(
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<TranslationIncludes>,
	Json(request): Json<CreateTranslationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_tr = request.to_insertable(session.data.profile_id);
	let translation = new_tr.insert(includes, &conn).await?;
	let response = TranslationResponse::from(translation);

	Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific translation with a given key and language
#[instrument(skip(pool))]
pub(crate) async fn get_translation(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
	Query(includes): Query<TranslationIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let translation = Translation::get_by_id(id, includes, &conn).await?;
	let response = TranslationResponse::from(translation);

	Ok((StatusCode::OK, Json(response)))
}

/// Update the translation with the given id.
#[instrument(skip(pool))]
pub(crate) async fn update_translation(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Query(includes): Query<TranslationIncludes>,
	Json(request): Json<UpdateTranslationRequest>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	let tr_update = request.to_insertable(session.data.profile_id);
	let updated_tr = tr_update.apply_to(id, includes, &conn).await?;
	let response = TranslationResponse::from(updated_tr);

	Ok((StatusCode::OK, Json(response)))
}

/// Delete the translation with the given id.
#[instrument(skip(pool))]
pub(crate) async fn delete_translation(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	Translation::delete_by_id(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
