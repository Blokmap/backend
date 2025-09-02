use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use image::{Image, ImageIncludes};
use permissions::{
	AuthorityPermissions,
	InstitutionPermissions,
	LocationPermissions,
	check_location_perms,
};
use utils::image::{delete_image, store_location_image};

use crate::schemas::BuildResponse;
use crate::schemas::image::{CreateOrderedImageRequest, ImageResponse};
use crate::schemas::location::LocationImageOrderUpdate;
use crate::{Config, Session};

#[instrument(skip(pool, config, data))]
pub async fn upload_location_image(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
	mut data: Multipart,
) -> Result<impl IntoResponse, Error> {
	check_location_perms(
		id,
		session.data.profile_id,
		LocationPermissions::ManageImages | LocationPermissions::Administrator,
		AuthorityPermissions::Administrator,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let image = CreateOrderedImageRequest::parse(&mut data).await?.into();
	let inserted_image =
		store_location_image(session.data.profile_id, id, image, &conn).await?;
	let response =
		inserted_image.build_response(ImageIncludes::default(), &config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

pub async fn reorder_location_images(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Query(includes): Query<ImageIncludes>,
	Path(id): Path<i32>,
	Json(new_order): Json<Vec<LocationImageOrderUpdate>>,
) -> Result<impl IntoResponse, Error> {
	// TODO: only allow reordering if the current images are approved
	// TODO: only allow reordering if {current_image_ids} =
	// {reordered_image_ids}

	check_location_perms(
		id,
		session.data.profile_id,
		LocationPermissions::ManageImages | LocationPermissions::Administrator,
		AuthorityPermissions::Administrator,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let new_order =
		new_order.into_iter().map(|o| o.to_insertable(id)).collect();
	let images = Image::reorder(id, new_order, includes, &conn).await?;

	let response: Vec<ImageResponse> = images
		.into_iter()
		.map(|i| i.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_location_image(
	State(pool): State<DbPool>,
	session: Session,
	Path((l_id, img_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	check_location_perms(
		l_id,
		session.data.profile_id,
		LocationPermissions::ManageImages | LocationPermissions::Administrator,
		AuthorityPermissions::Administrator,
		InstitutionPermissions::Administrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;
	delete_image(img_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
