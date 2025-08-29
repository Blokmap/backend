use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use profile::Profile;
use utils::image::{delete_image, store_profile_image};

use crate::Session;
use crate::schemas::image::CreateImageRequest;

#[instrument(skip(pool, data))]
pub async fn upload_profile_avatar(
	State(pool): State<DbPool>,
	session: Session,
	Path(p_id): Path<i32>,
	mut data: Multipart,
) -> Result<impl IntoResponse, Error> {
	if session.data.profile_id != p_id {
		return Err(Error::Forbidden);
	}

	let conn = pool.get().await?;

	let profile = Profile::get(p_id, &conn).await?;
	if let Some(img_id) = profile.primitive.avatar_image_id {
		delete_image(img_id, &conn).await?;
	}

	let image_request = CreateImageRequest::parse(&mut data).await?;
	let image = store_profile_image(p_id, image_request.into(), &conn).await?;

	Ok((StatusCode::CREATED, Json(image)))
}

#[instrument(skip(pool))]
pub async fn delete_profile_avatar(
	State(pool): State<DbPool>,
	session: Session,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	if session.data.profile_id != p_id && !session.data.is_admin {
		return Err(Error::Forbidden);
	}

	let conn = pool.get().await?;

	let profile = Profile::get(p_id, &conn).await?;
	let Some(img_id) = profile.primitive.avatar_image_id else {
		return Ok((StatusCode::NO_CONTENT, NoContent));
	};

	delete_image(img_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
