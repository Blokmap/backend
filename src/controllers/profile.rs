//! Controllers for [`Profile`]s

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error, RedisConn};
use image::ImageEncoder;
use image::codecs::webp::WebPEncoder;
use models::{
	Authority,
	AuthorityIncludes,
	Image,
	Location,
	LocationIncludes,
	NewImage,
	PrimitiveProfile,
	ProfileState,
	Reservation,
	ReservationIncludes,
	Review,
	UpdateProfile,
};
use uuid::Uuid;

use crate::image::{ImageOwner, generate_image_filepaths, resize_image};
use crate::mailer::Mailer;
use crate::schemas::authority::AuthorityResponse;
use crate::schemas::location::LocationResponse;
use crate::schemas::pagination::{PaginationOptions, PaginationResponse};
use crate::schemas::profile::{ProfileResponse, UpdateProfileRequest};
use crate::schemas::reservation::ReservationResponse;
use crate::schemas::review::ReviewLocationResponse;
use crate::{AdminSession, Config, Session};

/// Get all [`Profile`]s
#[instrument(skip(pool))]
pub async fn get_all_profiles(
	State(pool): State<DbPool>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<Json<PaginationResponse<Vec<ProfileResponse>>>, Error> {
	let conn = pool.get().await?;

	let (total, truncated, profiles) =
		PrimitiveProfile::get_all(p_opts.limit(), p_opts.offset(), &conn)
			.await?;

	let profiles: Vec<ProfileResponse> =
		profiles.into_iter().map(Into::into).collect();

	let paginated = p_opts.paginate(total, truncated, profiles);

	Ok(Json(paginated))
}

#[instrument(skip(pool))]
pub async fn get_current_profile(
	State(pool): State<DbPool>,
	session: Session,
) -> Result<Json<ProfileResponse>, Error> {
	let conn = pool.get().await?;

	let profile = PrimitiveProfile::get(session.data.profile_id, &conn).await?;

	Ok(Json(profile.into()))
}

#[instrument(skip(pool))]
pub async fn get_profile(
	State(pool): State<DbPool>,
	session: Session,
	Path(p_id): Path<i32>,
) -> Result<Json<ProfileResponse>, Error> {
	let conn = pool.get().await?;

	if !session.data.profile_is_admin && p_id != session.data.profile_id {
		return Err(Error::Forbidden);
	}

	let profile = PrimitiveProfile::get(p_id, &conn).await?;

	Ok(Json(profile.into()))
}

#[instrument(skip(pool, config, mailer))]
pub async fn update_current_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	session: Session,
	Json(update): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, Error> {
	let conn = pool.get().await?;

	let old_profile =
		PrimitiveProfile::get(session.data.profile_id, &conn).await?;

	let mut updated_profile = UpdateProfile::from(update)
		.apply_to(session.data.profile_id, &conn)
		.await?;

	if old_profile.pending_email != updated_profile.pending_email {
		let email_confirmation_token = Uuid::new_v4().to_string();

		updated_profile = updated_profile
			.set_email_confirmation_token(
				&email_confirmation_token,
				config.email_confirmation_token_lifetime,
				&conn,
			)
			.await?;

		mailer
			.send_confirm_email(
				&updated_profile,
				&email_confirmation_token,
				&config.frontend_url,
			)
			.await?;

		info!("set new pending email for profile {}", updated_profile.id);
	}

	Ok(Json(updated_profile.into()))
}

#[instrument(skip(pool, config, mailer))]
pub async fn update_profile(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	State(mailer): State<Mailer>,
	session: Session,
	Path(p_id): Path<i32>,
	Json(update): Json<UpdateProfileRequest>,
) -> Result<Json<ProfileResponse>, Error> {
	let conn = pool.get().await?;

	if !session.data.profile_is_admin && p_id != session.data.profile_id {
		return Err(Error::Forbidden);
	}

	let old_profile = PrimitiveProfile::get(p_id, &conn).await?;

	let mut updated_profile =
		UpdateProfile::from(update).apply_to(p_id, &conn).await?;

	if old_profile.pending_email != updated_profile.pending_email {
		let email_confirmation_token = Uuid::new_v4().to_string();

		updated_profile = updated_profile
			.set_email_confirmation_token(
				&email_confirmation_token,
				config.email_confirmation_token_lifetime,
				&conn,
			)
			.await?;

		mailer
			.send_confirm_email(
				&updated_profile,
				&email_confirmation_token,
				&config.frontend_url,
			)
			.await?;

		info!("set new pending email for profile {}", updated_profile.id);
	}

	Ok(Json(updated_profile.into()))
}

#[instrument(skip(pool))]
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

	let mut image_bytes = None;

	while let Some(field) = data.next_field().await? {
		if field.name().unwrap_or_default() != "image" {
			continue;
		}

		image_bytes = Some(field.bytes().await?);

		break;
	}

	let Some(image_bytes) = image_bytes else {
		return Err(Error::MissingRequestData("image".into()));
	};

	let (dst_image, dst_width, dst_height, dst_color) =
		resize_image(image_bytes)?;
	let (abs_filepath, rel_filepath) =
		generate_image_filepaths(p_id, ImageOwner::Profile)?;

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

	let image = PrimitiveProfile::insert_avatar(p_id, new_image, &conn).await?;

	Ok((StatusCode::CREATED, Json(image)))
}

#[instrument(skip(pool))]
pub async fn delete_profile_avatar(
	State(pool): State<DbPool>,
	session: Session,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	if session.data.profile_id != p_id && !session.data.profile_is_admin {
		return Err(Error::Forbidden);
	}

	let conn = pool.get().await?;

	let profile = PrimitiveProfile::get(p_id, &conn).await?;
	let Some(img_id) = profile.avatar_image_id else {
		return Ok((StatusCode::NO_CONTENT, NoContent));
	};

	// Delete the image record before the file to prevent dangling
	let image = Image::get_by_id(img_id, &conn).await?;
	Image::delete_by_id(img_id, &conn).await?;

	let filepath = PathBuf::from("/mnt/files").join(&image.file_path);
	std::fs::remove_file(filepath)?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

#[instrument(skip(pool))]
pub async fn disable_profile(
	State(pool): State<DbPool>,
	State(mut r_conn): State<RedisConn>,
	session: AdminSession,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let mut profile = PrimitiveProfile::get(profile_id, &conn).await?;

	profile.state = ProfileState::Disabled;
	profile.update(&conn).await?;

	Session::delete(profile_id, &mut r_conn).await?;

	info!("disabled profile {profile_id}");

	Ok(NoContent)
}

#[instrument(skip(pool))]
pub async fn activate_profile(
	State(pool): State<DbPool>,
	session: AdminSession,
	Path(profile_id): Path<i32>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;
	let mut profile = PrimitiveProfile::get(profile_id, &conn).await?;

	profile.state = ProfileState::Active;
	profile.update(&conn).await?;

	info!("disabled profile {profile_id}");

	Ok(NoContent)
}

#[instrument(skip(pool))]
pub async fn get_profile_locations(
	State(pool): State<DbPool>,
	Query(includes): Query<LocationIncludes>,
	Path(profile_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;
	let locations =
		Location::get_by_profile_id(profile_id, includes, &conn).await?;
	let response: Vec<LocationResponse> =
		locations.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_reservations(
	State(pool): State<DbPool>,
	Query(includes): Query<ReservationIncludes>,
	Path(profile_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let reservations =
		Reservation::for_profile(profile_id, includes, &conn).await?;
	let response: Vec<ReservationResponse> =
		reservations.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_authorities(
	State(pool): State<DbPool>,
	Query(includes): Query<AuthorityIncludes>,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let authorities = Authority::for_profile(p_id, includes, &conn).await?;
	let response: Vec<AuthorityResponse> =
		authorities.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_profile_reviews(
	State(pool): State<DbPool>,
	Path(p_id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let reviews = Review::for_profile(p_id, &conn).await?;
	let response: Vec<ReviewLocationResponse> =
		reviews.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}
