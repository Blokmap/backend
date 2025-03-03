use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DbPool;
use crate::error::Error;
use crate::models::{Language, NewTranslation, NewTranslations, Translation};

#[derive(Deserialize, Serialize)]
pub struct CreateTranslationResponse {
	pub key:         Uuid,
	pub translation: Translation,
}

/// Create and store a single translation in the database
#[instrument(skip(pool))]
pub async fn create_translation(
	State(pool): State<DbPool>,
	Json(translation): Json<NewTranslation>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let translation = translation.insert(conn).await?;
	let key = translation.key;

	Ok((
		StatusCode::CREATED,
		Json(CreateTranslationResponse { key, translation }),
	))
}

#[derive(Deserialize, Serialize)]
pub struct CreateTranslationsResponse {
	pub key:          Uuid,
	pub translations: Vec<Translation>,
}

/// Create and store a list of translation in the database
#[instrument(skip(pool))]
pub async fn create_translations(
	State(pool): State<DbPool>,
	Json(translations): Json<NewTranslations>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (key, translations) = translations.insert(conn).await?;

	Ok((
		StatusCode::CREATED,
		Json(CreateTranslationsResponse { key, translations }),
	))
}

/// Get a specific translation with a given key and language
#[instrument(skip(pool))]
pub async fn get_translation(
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
pub async fn get_translations(
	State(pool): State<DbPool>,
	Path(key): Path<Uuid>,
) -> Result<Json<Vec<Translation>>, Error> {
	let conn = pool.get().await?;

	let translations = Translation::get_by_key(key, conn).await?;

	Ok(Json(translations))
}

/// Delete the translation with the given key and language
#[instrument(skip(pool))]
pub async fn delete_translation(
	State(pool): State<DbPool>,
	Path((key, language)): Path<(Uuid, Language)>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;

	Translation::delete_by_key_and_language(key, language, conn).await?;

	Ok(NoContent)
}

/// Delete all translations with a given key
#[instrument(skip(pool))]
pub async fn delete_translations(
	State(pool): State<DbPool>,
	Path(key): Path<Uuid>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;

	Translation::delete_by_key(key, conn).await?;

	Ok(NoContent)
}
