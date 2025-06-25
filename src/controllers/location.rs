//! Controllers for [`Location`]s

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use image::ImageEncoder;
use image::codecs::webp::WebPEncoder;
use models::{
	AuthorityPermissions,
	Image,
	Location,
	LocationFilter,
	LocationIncludes,
	NewImage,
	PrimitiveOpeningTime,
	Tag,
	TimeFilter,
};
use rayon::prelude::*;

use crate::image::{ImageOwner, generate_image_filepaths, resize_image};
use crate::schemas::location::{
	CreateLocationRequest,
	LocationResponse,
	RejectLocationRequest,
	UpdateLocationRequest,
};
use crate::schemas::pagination::PaginationOptions;
use crate::schemas::tag::SetLocationTagsRequest;
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
			let (abs_filepath, rel_filepath) =
				generate_image_filepaths(id, ImageOwner::Location)?;

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
	Path((l_id, img_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= AuthorityPermissions::location_admin_or(
		session.data.profile_id,
		l_id,
		AuthorityPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	// Delete the image record before the file to prevent dangling
	let image = Image::get_by_id(l_id, &conn).await?;
	Image::delete_by_id(l_id, &conn).await?;

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

	let (total, locations) = Location::search(
		loc_filter,
		time_filter,
		includes,
		limit,
		offset,
		&conn,
	)
	.await?;

	let l_ids = locations.iter().map(|l| l.location.id).collect::<Vec<_>>();

	let (times, tags, imgs) = tokio::join!(
		PrimitiveOpeningTime::get_for_locations(l_ids.clone(), &conn),
		Tag::get_for_locations(l_ids.clone(), &conn),
		Image::get_for_locations(l_ids, &conn),
	);

	let times = times?;
	let tags = tags?;
	let imgs = imgs?;

	let locations = Location::group(locations, &times, &tags, &imgs);

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

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= AuthorityPermissions::location_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
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

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= AuthorityPermissions::location_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::ApproveLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

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

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= AuthorityPermissions::location_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::ApproveLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	Location::reject_by(id, session.data.profile_id, request.reason, &conn)
		.await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

pub async fn set_location_tags(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Json(data): Json<SetLocationTagsRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= AuthorityPermissions::location_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	Tag::bulk_set(id, data.tags, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

/// Delete a location from the database.
#[instrument(skip(pool))]
pub(crate) async fn delete_location(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= AuthorityPermissions::location_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::DeleteLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	if !can_manage {
		return Err(Error::Forbidden);
	}

	Location::delete_by_id(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
