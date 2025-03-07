//! Controllers for [`Translation`]s

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use uuid::Uuid;

use super::schema::translation::{
	BulkTranslationsRequest,
	BulkTranslationsResponse,
	TranslationRequest,
};
use crate::DbPool;
use crate::error::Error;
use crate::models::{Language, NewTranslation, Translation};

/// Create and store a single translation in the database
#[instrument(skip(pool))]
pub(crate) async fn create_translation(
	State(pool): State<DbPool>,
	Json(translation): Json<TranslationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let translation: NewTranslation = translation.into();

	let translation = translation.insert(conn).await?;

	Ok((StatusCode::CREATED, Json(translation)))
}

/// Create and store a list of translation in the database
#[instrument(skip(pool))]
pub(crate) async fn create_bulk_translations(
	State(pool): State<DbPool>,
	Json(bulk): Json<BulkTranslationsRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (_, translations) =
		NewTranslation::bulk_insert(bulk.into(), &conn).await?;

	let translations = translations
		.into_iter()
		.map(|translation| (translation.language, translation))
		.collect();

	Ok((StatusCode::CREATED, Json(BulkTranslationsResponse(translations))))
}

/// Get a specific translation with a given key and language
#[instrument(skip(pool))]
pub(crate) async fn get_translation(
	State(pool): State<DbPool>,
	Path((key, language)): Path<(Uuid, Language)>,
) -> Result<Json<Translation>, Error> {
	let conn = pool.get().await?;

	let translation =
		Translation::get_by_key_and_language(key, language, conn).await?;

	Ok(Json(translation))
}

/// Get all translations with a given key
#[instrument(skip(pool))]
pub(crate) async fn get_bulk_translations(
	State(pool): State<DbPool>,
	Path(key): Path<Uuid>,
) -> Result<Json<Vec<Translation>>, Error> {
	let conn = pool.get().await?;

	let translations = Translation::get_by_key(key, conn).await?;

	Ok(Json(translations))
}

/// Delete the translation with the given key and language
#[instrument(skip(pool))]
pub(crate) async fn delete_translation(
	State(pool): State<DbPool>,
	Path((key, language)): Path<(Uuid, Language)>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;

	Translation::delete_by_key_and_language(key, language, conn).await?;

	Ok(NoContent)
}

/// Delete all translations with a given key
#[instrument(skip(pool))]
pub(crate) async fn delete_bulk_translations(
	State(pool): State<DbPool>,
	Path(key): Path<Uuid>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;

	Translation::delete_by_key(key, conn).await?;

	Ok(NoContent)
}
