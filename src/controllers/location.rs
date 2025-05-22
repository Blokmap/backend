//! Controllers for [`Location`]s

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use axum::{Extension, Json};

use crate::DbPool;
use crate::error::Error;
use crate::models::{
	FilledLocation,
	Location,
	LocationFilter,
	NewLocation,
	ProfileId,
};
use crate::schemas::location::{
	CreateLocationRequest,
	LocationResponse,
	UpdateLocationRequest,
};

/// Create a new location in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_location(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Json(request): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let request = NewLocation {
		name:           request.name,
		description_id: request.description_id,
		excerpt_id:     request.excerpt_id,
		seat_count:     request.seat_count,
		is_reservable:  request.is_reservable,
		is_visible:     request.is_visible,
		street:         request.street,
		number:         request.number,
		zip:            request.zip,
		city:           request.city,
		province:       request.province,
		latitude:       request.latitude,
		longitude:      request.longitude,
		created_by_id:  *profile_id,
	};

	let location = request.insert(&conn).await?;
	let location = Location::get_by_id(location.id, &conn).await?;

	let response = LocationResponse::from(location);

	Ok((StatusCode::CREATED, Json(response)))
}

/// Get a location from the database.
#[instrument(skip(pool))]
pub(crate) async fn get_location(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let result = Location::get_by_id(id, &conn).await?;

	let response = LocationResponse::from(result);

	Ok((StatusCode::OK, Json(response)))
}

/// Get all positions of locations from the database.
#[instrument(skip(pool))]
pub(crate) async fn get_location_positions(
	State(pool): State<DbPool>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let positions = Location::get_latlng_positions(&conn).await?;

	Ok((StatusCode::OK, Json(positions)))
}

/// Search all locations from the database on given latlng bounds.
/// The latlng bounds include the southwestern and northeastern corners.
/// The southwestern corner is the minimum latitude and longitude, and the
/// northeastern corner is the maximum latitude and longitude.
///
/// /location/{id}
/// /location/{id}?distance=51.123-3.456-5
/// /location/{id}?name=KCGG UZ Gent
/// /location/{id}?has_reservations=1/0
/// /location/{id}?open_on=2025-01-01
#[instrument(skip(pool))]
pub(crate) async fn get_locations(
	State(pool): State<DbPool>,
	Query(filter): Query<LocationFilter>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let all_dist = filter.distance.is_some()
		&& filter.center_lat.is_some()
		&& filter.center_lng.is_some();

	let any_dist = filter.distance.is_some()
		|| filter.center_lat.is_some()
		|| filter.center_lng.is_some();

	if all_dist != any_dist {
		return Err(Error::ValidationError(
			"expected all of distance, centerLat, centerLng to be set".into(),
		));
	}

	let all_bounds = filter.north_east_lat.is_some()
		&& filter.north_east_lng.is_some()
		&& filter.south_west_lat.is_some()
		&& filter.south_west_lng.is_some();

	let any_bounds = filter.north_east_lat.is_some()
		|| filter.north_east_lng.is_some()
		|| filter.south_west_lat.is_some()
		|| filter.south_west_lng.is_some();

	if all_bounds != any_bounds {
		return Err(Error::ValidationError(
			"expected all of northEastLat, northEastLng, southWestLat, \
			 southWestLng to be set"
				.into(),
		));
	}

	let locations = FilledLocation::search(filter, &conn).await?;

	Ok((StatusCode::OK, Json(locations)))
}

/// Update a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn update_location(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Path(id): Path<i32>,
	Json(request): Json<UpdateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (location, ..) = Location::get_by_id(id, &conn).await?;

	if *profile_id != location.created_by_id {
		return Err(Error::Forbidden);
	}

	let location = request.location.update(id, &conn).await?;

	let response = LocationResponse::from(location);

	Ok((StatusCode::OK, Json(response)))
}

/// Approve a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn approve_location(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	Location::approve_by(id, *profile_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

/// Delete a location from the database.
#[instrument(skip(pool))]
pub(crate) async fn delete_location(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	Location::delete_by_id(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
