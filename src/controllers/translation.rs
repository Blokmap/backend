use axum::Json;
use axum::extract::State;
use serde::Serialize;
use uuid::Uuid;

use crate::DbPool;
use crate::error::Error;
use crate::models::{Language, NewTranslation, NewTranslations, Translation};

/// Get all translations with a given key
pub async fn get_translations(
	key: Uuid,
	State(pool): State<DbPool>,
) -> Result<Json<Vec<Translation>>, Error> {
	let conn = pool.get().await?;

	let translations = Translation::get_by_key(key, conn).await?;

	Ok(Json(translations))
}

/// Get a specific translation with a given key and language
pub async fn get_translation(
	key: Uuid,
	language: Language,
	State(pool): State<DbPool>,
) -> Result<Json<Translation>, Error> {
	let conn = pool.get().await?;

	let translation = Translation::get_by_key_and_language(key, language, conn).await?;

	Ok(Json(translation))
}

#[derive(Serialize)]
pub struct CreateTranslationsResponse {
	key:          Uuid,
	translations: Vec<Translation>,
}

/// Create and store a list of translation in the database
pub async fn create_translations(
	translations: NewTranslations,
	State(pool): State<DbPool>,
) -> Result<Json<CreateTranslationsResponse>, Error> {
	let conn = pool.get().await?;

	let (key, translations) = translations.insert(conn).await?;

	Ok(Json(CreateTranslationsResponse { key, translations }))
}

#[derive(Serialize)]
pub struct CreateTranslationResponse {
	key:         Uuid,
	translation: Translation,
}

/// Create and store a single translation in the database
pub async fn create_translation(
	translation: NewTranslation,
	State(pool): State<DbPool>,
) -> Result<Json<CreateTranslationResponse>, Error> {
	let conn = pool.get().await?;

	let translation = translation.insert(conn).await?;
	let key = translation.key;

	Ok(Json(CreateTranslationResponse { key, translation }))
}
