//! Controllers for [`Location`]s

use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::PathBuf;

use axum::body::Bytes;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use axum::{Extension, Json};
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::codecs::webp::WebPEncoder;
use image::{ColorType, ImageEncoder, ImageReader};
use rayon::prelude::*;
use uuid::Uuid;

use crate::DbPool;
use crate::error::Error;
use crate::models::{Location, LocationFilter, NewLocation, NewLocationImage, ProfileId};
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

#[inline]
fn resize_image(
	bytes: Bytes,
) -> Result<(Image<'static>, u32, u32, ColorType), Error> {
	let image_reader =
		ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;

	let src_image = image_reader.decode()?;

	// Set width to 1024 but scale height to preserve aspect ratio
	#[allow(clippy::cast_precision_loss)]
	let src_ratio = src_image.height() as f32 / src_image.width() as f32;
	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_sign_loss)]
	let dst_height = (1024.0 * src_ratio) as u32;
	let dst_width = 1024;

	let mut dst_image =
		Image::new(dst_width, dst_height, src_image.pixel_type().unwrap());

	let mut resizer = Resizer::new();
	resizer.resize(&src_image, &mut dst_image, None)?;

	Ok((dst_image, dst_width, dst_height, src_image.color()))
}

/// Generate both an absolute and relative filepath for a new image
///
/// The absolute path is used for writing to disk, the relative path is used
/// by the API
#[inline]
fn generate_image_filepaths(id: i32) -> Result<(PathBuf, PathBuf), Error> {
	let image_uuid = Uuid::new_v4().to_string();
	let rel_filepath =
		PathBuf::from(id.to_string()).join(image_uuid).with_extension("webp");

	let abs_filepath = PathBuf::from("/mnt/files").join(&rel_filepath);

	// Ensure all parent directories exist
	let prefix = abs_filepath.parent().unwrap();
	std::fs::create_dir_all(prefix)?;

	Ok((abs_filepath, rel_filepath))
}

#[instrument(skip(pool, data))]
pub(crate) async fn upload_location_image(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Path(id): Path<i32>,
	mut data: Multipart,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let mut image_bytes = vec![];
	while let Some(field) = data.next_field().await? {
		if field.name().unwrap_or_default() != "image" {
			continue;
		}

		image_bytes.push(field.bytes().await?);
	}

	let new_images = image_bytes
		.into_par_iter()
		.map(|bytes| {
			let (dst_image, dst_width, dst_height, dst_color) =
				resize_image(bytes)?;
			let (abs_filepath, rel_filepath) = generate_image_filepaths(id)?;

			let mut file = BufWriter::new(File::create(&abs_filepath)?);

			WebPEncoder::new_lossless(&mut file).write_image(
				dst_image.buffer(),
				dst_width,
				dst_height,
				dst_color.into(),
			)?;

			file.flush()?;

			let new_image = NewLocationImage {
				location_id: id,
				file_path:   rel_filepath.to_string_lossy().into_owned(),
				uploaded_by: *profile_id,
			};

			Ok(new_image)
		})
		.collect::<Result<Vec<NewLocationImage>, Error>>()?;

	let images = NewLocationImage::bulk_insert(new_images, &conn).await?;

	let image_paths: Vec<_> = images.into_iter().map(|i| i.file_path).collect();

	Ok((StatusCode::CREATED, Json(image_paths)))
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

	let locations = Location::search(filter, &conn).await?;
	let locations: Vec<_> =
		locations.into_iter().map(LocationResponse::from).collect();

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
