//! Controllers for [`Location`]s

use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::PathBuf;

use axum::body::Bytes;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use axum::{Extension, Json};
use common::{DbPool, Error};
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::codecs::webp::WebPEncoder;
use image::{ColorType, ImageEncoder, ImageReader};
use models::{
	Image as DbImage,
	Location,
	LocationFilter,
	NewImage,
	PaginationOptions,
};
use rayon::prelude::*;
use uuid::Uuid;

use crate::ProfileId;
use crate::schemas::location::{
	CreateLocationRequest,
	LocationResponse,
	RejectLocationRequest,
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

	let loc_data = request.location.to_insertable(*profile_id);
	let desc_data = request.description.to_insertable(*profile_id);
	let exc_data = request.excerpt.to_insertable(*profile_id);

	let records = Location::new(loc_data, desc_data, exc_data, &conn).await?;

	let response = LocationResponse::from(records);

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

	let abs_filepath =
		PathBuf::from("/mnt/files/locations").join(&rel_filepath);

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

			let new_image = NewImage {
				file_path:   rel_filepath.to_string_lossy().into_owned(),
				uploaded_by: *profile_id,
			};

			Ok(new_image)
		})
		.collect::<Result<Vec<NewImage>, Error>>()?;

	let images = Location::insert_images(id, new_images, &conn).await?;

	let image_paths: Vec<_> = images.into_iter().map(|i| i.file_path).collect();

	Ok((StatusCode::CREATED, Json(image_paths)))
}

#[instrument(skip(pool))]
pub async fn delete_location_image(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Path(id): Path<i32>,
	Path(image_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// Delete the image record before the file prevent dangling
	let image = DbImage::get_by_id(id, &conn).await?;
	DbImage::delete_by_id(id, &conn).await?;

	let filepath = PathBuf::from("/mnt/files/locations").join(&image.file_path);
	std::fs::remove_file(filepath)?;

	Ok((StatusCode::NO_CONTENT, NoContent))
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

#[instrument(skip(pool))]
pub(crate) async fn get_locations(
	State(pool): State<DbPool>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (total, locations) = Location::get_all(p_opts, &conn).await?;
	let locations: Vec<LocationResponse> =
		locations.into_iter().map(Into::into).collect();

	let paginated = p_opts.paginate(total, locations);

	Ok((StatusCode::OK, Json(paginated)))
}

/// Search all locations from the database on given latlng bounds.
/// The latlng bounds include the southwestern and northeastern corners.
/// The southwestern corner is the minimum latitude and longitude, and the
/// northeastern corner is the maximum latitude and longitude.
#[instrument(skip(pool))]
pub(crate) async fn search_locations(
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

	if Some(*profile_id) != location.created_by {
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

/// Reject a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn reject_location(
	State(pool): State<DbPool>,
	Extension(profile_id): Extension<ProfileId>,
	Path(id): Path<i32>,
	Json(request): Json<RejectLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	Location::reject_by(id, *profile_id, request.reason, &conn).await?;

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
