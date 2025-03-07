//! Controllers for [`Location`]s

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use super::schema::location::{LocationRequest, LocationResponse};
use crate::DbPool;
use crate::error::Error;
use crate::models::NewTranslation;

/// Create a new location in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_location(
	State(pool): State<DbPool>,
	Json(request): Json<LocationRequest>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Generate escription and excerpt translations.
	let (description_key, description_translations) =
		NewTranslation::bulk_insert(request.description.clone().into(), &conn)
			.await?;

	let (excerpt_key, excert_translations) =
		NewTranslation::bulk_insert(request.excerpt.clone().into(), &conn)
			.await?;

	// Generate new location.
	let location = request
		.to_new_location(excerpt_key, description_key)
		.insert(&conn)
		.await?;

	let response = LocationResponse::from_location(
		location,
		description_translations,
		excert_translations,
	);

	Ok((StatusCode::CREATED, Json(response)))
}
