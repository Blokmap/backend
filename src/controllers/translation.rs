//! Controllers for [`Translation`]s

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use axum::{Extension, Json};
use common::{DbPool, Error};
use models::{NewTranslation, Translation, UpdateTranslation};

use crate::ProfileId;
use crate::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
	UpdateTranslationRequest,
};

/// Create and store a single translation in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_translation(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Json(request): Json<CreateTranslationRequest>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	let translation = NewTranslation {
		nl:         request.nl,
		en:         request.en,
		fr:         request.fr,
		de:         request.de,
		created_by: *profile_id,
	};

	// Insert the translation into the database.
	let translation = translation.insert(&conn).await?;

	// Return a response with the created translation.
	let response = TranslationResponse::from(translation);

	Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific translation with a given key and language
#[instrument(skip(pool))]
pub(crate) async fn get_translation(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Get the translation from the database.
	let translation = Translation::get_by_id(id, &conn).await?;

	// Return a response with the translation.
	let response = TranslationResponse::from(translation);

	Ok((StatusCode::OK, Json(response)))
}

/// Delete the translation with the given id.
#[instrument(skip(pool))]
pub(crate) async fn delete_translation(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Delete the translation from the database.
	Translation::delete_by_id(id, &conn).await?;

	// Return a response with no content.
	Ok((StatusCode::NO_CONTENT, NoContent))
}

/// Update the translation with the given id.
#[instrument(skip(pool))]
pub(crate) async fn update_translation(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Path(id): Path<i32>,
	Json(request): Json<UpdateTranslationRequest>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	let translation = UpdateTranslation {
		nl:         request.nl,
		en:         request.en,
		fr:         request.fr,
		de:         request.de,
		updated_by: *profile_id,
	};

	// Update the translation in the database.
	let translation = translation.update(id, &conn).await?;

	// Return a response with the updated translation.
	let response = TranslationResponse::from(translation);

	Ok((StatusCode::OK, Json(response)))
}
