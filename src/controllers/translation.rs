//! Controllers for [`Translation`]s

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DbPool;
use crate::error::Error;
use crate::models::{InsertableTranslation, Language, Translation};

/// The data needed to make a new [`Translation`]
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTranslationRequest {
	pub language: Language,
	pub key:      Option<Uuid>,
	pub text:     String,
}

impl From<CreateTranslationRequest> for InsertableTranslation {
	fn from(value: CreateTranslationRequest) -> Self {
		InsertableTranslation {
			language: value.language,
			// Unwrap is safe as invald UUIDs are caught by serde validation
			key:      value.key.unwrap_or_else(Uuid::new_v4),
			text:     value.text,
		}
	}
}

/// The data returned when making a new [`Translation`]
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTranslationResponse {
	pub key:             Uuid,
	pub new_translation: Translation,
}

/// Create and store a single translation in the database
#[instrument(skip(pool))]
pub(crate) async fn create_translation(
	State(pool): State<DbPool>,
	Json(translation): Json<CreateTranslationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let translation: InsertableTranslation = translation.into();

	let new_translation = translation.insert(conn).await?;
	let key = new_translation.key;

	Ok((
		StatusCode::CREATED,
		Json(CreateTranslationResponse { key, new_translation }),
	))
}

/// The data returned when making a list of new [`Translation`]s
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateBulkTranslationsRequest {
	pub key:          Option<Uuid>,
	pub translations: Vec<CreateTranslationRequest>,
}

/// The data returned when making a list of new [`Translation`]s
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateBulkTranslationsResponse {
	pub key:              Uuid,
	pub new_translations: Vec<Translation>,
}

/// Create and store a list of translation in the database
#[instrument(skip(pool))]
pub(crate) async fn create_bulk_translations(
	State(pool): State<DbPool>,
	Json(bulk): Json<CreateBulkTranslationsRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// Unwrap is safe as invald UUIDs are caught by serde validation
	let key = bulk.key.unwrap_or_else(Uuid::new_v4);

	let insertables: Vec<InsertableTranslation> =
		bulk.translations.into_iter().map(Into::into).collect();

	let new_translations =
		InsertableTranslation::bulk_insert(insertables, conn).await?;

	Ok((
		StatusCode::CREATED,
		Json(CreateBulkTranslationsResponse { key, new_translations }),
	))
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
