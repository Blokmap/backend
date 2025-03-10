//! Controllers for [`Location`]s

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::DbPool;
use crate::error::Error;
use crate::models::Location;
use crate::schemas::location::{
	CreateLocationRequest,
	LocationResponse,
	UpdateLocationRequest,
};

/// Create a new location in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_location(
	State(pool): State<DbPool>,
	Json(request): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Insert the location into the database.
	let location = request.location.insert(&conn).await?;

	// Return the newly generated location.
	let response = LocationResponse::from(location);

	Ok((StatusCode::CREATED, Json(response)))
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
	let location = Location::get_by_id(id, &conn).await?;

	// Return the location response.
	let response = LocationResponse::from(location);

	Ok((StatusCode::OK, Json(response)))
}

pub(crate) async fn get_locations(
	State(pool): State<DbPool>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Get the locations from the database.
	let locations: Vec<Location> = Location::get_all(&conn).await?;

	// Return the locations response.
	let response: Vec<LocationResponse> =
		locations.into_iter().map(LocationResponse::from).collect();

	// Return the locations.
	Ok((StatusCode::OK, Json(response)))
}

/// Update a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn update_location(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
	Json(request): Json<UpdateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	// Get a connection from the pool.
	let conn = pool.get().await?;

	// Update the location in the database.
	let location = request.location.update(id, &conn).await?;

	// Return the updated location.
	let response = LocationResponse::from(location);

	Ok((StatusCode::OK, Json(response)))
}

/// Delete a location from the database.
#[instrument(skip(pool))]
pub(crate) async fn delete_location(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
    // Get a connection from the pool.
    let conn = pool.get().await?;

    // Delete the location from the database.
    Location::delete_by_id(id, &conn).await?;

    // Return a response with no content.
    Ok((StatusCode::NO_CONTENT, axum::response::NoContent))
}