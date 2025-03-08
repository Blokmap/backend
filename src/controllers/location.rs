//! Controllers for [`Location`]s

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use super::schema::location::{LocationRequest, LocationResponse};
use super::schema::translation::BulkTranslationsResponse;
use crate::DbPool;
use crate::error::Error;
use crate::models::{Location, NewLocation, Translation};

/// Create a new location in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_location(
	State(pool): State<DbPool>,
	Json(request): Json<LocationRequest>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Generate new location.
	let location: Location = NewLocation::insert(request.into(), &conn).await?;

	// Return the newly generated location.
	Ok((StatusCode::CREATED, Json(location)))
}

/// Get a location from the database.
#[instrument(skip(pool))]
pub(crate) async fn get_location(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Get the location from the database.
	let location: Location = Location::get_by_id(id, &conn).await?;

	// Get the translations for the location.
	let description: BulkTranslationsResponse =
		Translation::get_by_key(location.description_key.clone(), &conn)
			.await?
			.into();

	let excerpt: BulkTranslationsResponse =
		Translation::get_by_key(location.excerpt_key.clone(), &conn)
			.await?
			.into();

	// Return the location response.
	let response = LocationResponse::from(location);

	Ok(Json(LocationResponse {
		description: description.into(),
		excerpt: excerpt.into(),
		..response
	}))
}

pub(crate) async fn get_locations(
    State(pool): State<DbPool>,
) -> Result<impl IntoResponse, Error> {
    // Get a connection from the pool.
    let conn = pool.get().await?;

    // Get the locations from the database.
    let locations: Vec<Location> = Location::get_all(&conn).await?;

    // Return the locations.
    Ok(Json(locations))
}

// Search for locations in the by latitude and longitude bounds.
// #[instrument(skip(pool))]
// pub(crate) async fn search_locations(
// 	State(pool): State<DbPool>,
// 	Path((southwest, northeast)): Path<((f64, f64), (f64, f64))>,
// ) -> Result<impl IntoResponse, Error> {
// 	// Get a connection from the pool.
// 	let conn = pool.get().await.unwrap();

// 	// Get the locations from the database.
// 	let locations: Vec<Location> =
// 		Location::get_by_bounds(southwest, northeast, &conn).await.unwrap();

// 	Ok(Json(locations))
// }
