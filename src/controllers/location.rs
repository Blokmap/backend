//! Controllers for [`Location`]s

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::PathBuf;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::codecs::webp::WebPEncoder;
use image::{ColorType, ImageEncoder, ImageReader};
use models::{
	Image as DbImage,
	Location,
	LocationFilter,
	LocationIncludes,
	NewImage,
	OpeningTime,
	TimeFilter,
};
use rayon::prelude::*;
use uuid::Uuid;

use crate::schemas::location::{
	CreateLocationRequest,
	LocationResponse,
	RejectLocationRequest,
	UpdateLocationRequest,
};
use crate::schemas::pagination::PaginationOptions;
use crate::{AdminSession, Session};

/// Create a new location in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_location(
	State(pool): State<DbPool>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Json(request): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let new_location = request.to_insertable(session.data.profile_id);
	let records = new_location.insert(includes, &conn).await?;
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
	session: Session,
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
				uploaded_by: session.data.profile_id,
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
	session: Session,
	Path(id): Path<i32>,
	Path(image_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: check permission

	// Delete the image record before the file to prevent dangling
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
	Query(includes): Query<LocationIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let result = Location::get_by_id(id, includes, &conn).await?;
	let response = LocationResponse::from(result);

	Ok((StatusCode::OK, Json(response)))
}

/// Search all locations from the database on given latlng bounds.
/// The latlng bounds include the southwestern and northeastern corners.
/// The southwestern corner is the minimum latitude and longitude, and the
/// northeastern corner is the maximum latitude and longitude.
#[instrument(skip(pool))]
pub(crate) async fn search_locations(
	State(pool): State<DbPool>,
	Query(time_filter): Query<TimeFilter>,
	Query(loc_filter): Query<LocationFilter>,
	Query(includes): Query<LocationIncludes>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	#[allow(clippy::cast_sign_loss)]
	#[allow(clippy::cast_possible_truncation)]
	let limit = p_opts.limit() as usize;
	#[allow(clippy::cast_sign_loss)]
	#[allow(clippy::cast_possible_truncation)]
	let offset = p_opts.offset() as usize;

	// let (loc_result, time_result) = tokio::join!(
	// 	Location::search(loc_filter, time_filter, includes, limit, offset,
	// &conn), 	OpeningTime::search(time_filter, &conn),
	// );

	// let (total, locations) = loc_result?;
	// let times = time_result?;

	let (total, locations) = Location::search(
		loc_filter,
		time_filter,
		includes,
		limit,
		offset,
		&conn,
	)
	.await?;

	let location_ids =
		locations.iter().map(|l| l.location.id).collect::<Vec<_>>();

	let times = OpeningTime::skibidi(location_ids, &conn).await?;

	// let locations = locations
	// 	.into_par_iter()
	// 	.filter(|l| time_location_ids.contains(&l.location.id))
	// 	.collect::<Vec<_>>();

	let mut id_map = HashMap::new();

	for loc in locations {
		let loc_id = loc.location.id;
		let entry = id_map.entry(loc).or_insert(vec![]);

		for time in &times {
			if time.location_id == loc_id {
				entry.push(time.clone());
			}
		}
	}

	let locations = id_map.into_iter().collect::<Vec<_>>();

	let locations: Vec<LocationResponse> =
		locations.into_iter().map(Into::into).collect();
	#[allow(clippy::cast_possible_wrap)]
	let total = total as i64;

	let paginated = p_opts.paginate(total, locations);

	Ok((StatusCode::OK, Json(paginated)))
}

/// Update a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn update_location(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Query(includes): Query<LocationIncludes>,
	Json(request): Json<UpdateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let perm_includes =
		LocationIncludes { created_by: true, ..Default::default() };
	let (location, ..) = Location::get_by_id(id, perm_includes, &conn).await?;

	// TODO: check permissions properly

	#[allow(clippy::collapsible_if)]
	if let Some(Some(creator)) = location.created_by {
		if creator.id != session.data.profile_id {
			return Err(Error::Forbidden);
		}
	}

	let loc_update = request.to_insertable(session.data.profile_id);
	let updated_loc = loc_update.apply_to(id, includes, &conn).await?;
	let response = LocationResponse::from(updated_loc);

	Ok((StatusCode::OK, Json(response)))
}

/// Approve a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn approve_location(
	State(pool): State<DbPool>,
	session: AdminSession,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	Location::approve_by(id, session.data.profile_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

/// Reject a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn reject_location(
	State(pool): State<DbPool>,
	session: AdminSession,
	Path(id): Path<i32>,
	Json(request): Json<RejectLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	Location::reject_by(id, session.data.profile_id, request.reason, &conn)
		.await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

/// Delete a location from the database.
#[instrument(skip(pool))]
pub(crate) async fn delete_location(
	State(pool): State<DbPool>,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: check permissions

	Location::delete_by_id(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
