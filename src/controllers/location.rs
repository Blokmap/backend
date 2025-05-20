//! Controllers for [`Location`]s

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use axum::{Extension, Json};

use crate::DbPool;
use crate::error::Error;
use crate::models::{
	Bounds,
	Location,
	NewLocation,
	NewLocationImage,
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

#[instrument(skip(pool, file))]
pub(crate) async fn upload_location_image(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Path(id): Path<i32>,
	mut file: Multipart,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	while let Some(field) = file.next_field().await? {
		let new_image =
			NewLocationImage { location_id: id, uploaded_by: *profile_id };

		let image = new_image.insert(&conn).await?;

		let extension = match field.content_type() {
			Some("image/png") => "png",
			Some("image/jpg" | "image/jpeg") => "jpg",
			Some("image/webp") => "webp",
			_ => "",
		};

		let bytes = field.bytes().await?;

		let filepath = PathBuf::from("/mnt/files")
			.join(id.to_string())
			.join(image.id.to_string())
			.with_extension(extension);

		let prefix = filepath.parent().unwrap();
		std::fs::create_dir_all(prefix)?;

		let mut file = File::create(&filepath)?;
		file.write_all(&bytes)?;
	}

	Ok(StatusCode::CREATED)
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
	// Get a connection from the pool.
	let conn = pool.get().await?;

	let positions = Location::get_latlng_positions(&conn).await?;

	Ok((StatusCode::OK, Json(positions)))
}

/// Search all locations from the database on given latlng bounds.
/// The latlng bounds include the southwestern and northeastern corners.
/// The southwestern corner is the minimum latitude and longitude, and the
/// northeastern corner is the maximum latitude and longitude.
#[instrument(skip(pool))]
pub(crate) async fn get_locations(
	State(pool): State<DbPool>,
	Query(bounds): Query<Bounds>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let locations = Location::get_all(bounds, &conn).await?;

	let response: Vec<LocationResponse> =
		locations.into_iter().map(LocationResponse::from).collect();

	Ok((StatusCode::OK, Json(response)))
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
