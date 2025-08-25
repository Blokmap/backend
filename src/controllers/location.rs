//! Controllers for [`Location`]s

use authority::AuthorityPermissions;
use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use image::Image;
use location::{
	Location,
	LocationFilter,
	LocationIncludes,
	LocationPermissions,
	Point,
};
use opening_time::{OpeningTime, OpeningTimeIncludes, TimeFilter};
use tag::Tag;
use utils::image::{delete_image, store_location_image};
use validator::Validate;

use crate::schemas::BuildResponse;
use crate::schemas::image::{CreateOrderedImageRequest, ImageResponse};
use crate::schemas::location::{
	CreateLocationMemberRequest,
	CreateLocationRequest,
	LocationImageOrderUpdate,
	LocationResponse,
	NearestLocationResponse,
	RejectLocationRequest,
	UpdateLocationMemberRequest,
	UpdateLocationRequest,
};
use crate::schemas::pagination::PaginationOptions;
use crate::schemas::profile::ProfilePermissionsResponse;
use crate::schemas::tag::SetLocationTagsRequest;
use crate::{AdminSession, Config, Session};

/// Create a new location in the database.
#[instrument(skip(pool))]
pub(crate) async fn create_location(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Query(includes): Query<LocationIncludes>,
	Json(request): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	request.validate()?;

	let new_location = request.to_insertable(session.data.profile_id);
	let records = new_location.insert(includes, &conn).await?;
	let response: LocationResponse = records.build_response(&config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool, config, data))]
pub async fn upload_location_image(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
	mut data: Multipart,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: permissions
	let profile_id = session.data.profile_id;

	let image = CreateOrderedImageRequest::parse(&mut data).await?.into();
	let inserted_image =
		store_location_image(profile_id, id, image, &conn).await?;
	let response: ImageResponse = inserted_image.build_response(&config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

pub async fn reorder_location_images(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
	Json(new_order): Json<Vec<LocationImageOrderUpdate>>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	// TODO: only allow reordering if the current images are approved
	// TODO: only allow reordering if {current_image_ids} =
	// {reordered_image_ids}

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= Location::owner_or_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	let new_order =
		new_order.into_iter().map(|o| o.to_insertable(id)).collect();
	let images = Image::reorder(id, new_order, &conn).await?;

	let response: Vec<ImageResponse> = images
		.into_iter()
		.map(|i| i.build_response(&config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
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

	can_manage |= Location::owner_or_admin_or(
		session.data.profile_id,
		l_id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	delete_image(img_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

/// Get a location from the database.
#[instrument(skip(pool))]
pub(crate) async fn get_location(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Path(id): Path<i32>,
	Query(includes): Query<LocationIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let result = Location::get_by_id(id, includes, &conn).await?;
	let response: LocationResponse = result.build_response(&config)?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub(crate) async fn get_nearest_location(
	State(pool): State<DbPool>,
	Query(point): Query<Point>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let info = Location::get_nearest(point, &conn).await?;
	let res: NearestLocationResponse = info.into();

	Ok((StatusCode::OK, Json(res)))
}

/// Search all locations from the database on given latlng bounds.
/// The latlng bounds include the southwestern and northeastern corners.
/// The southwestern corner is the minimum latitude and longitude, and the
/// northeastern corner is the maximum latitude and longitude.
#[instrument(skip(pool))]
pub(crate) async fn search_locations(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	Query(time_filter): Query<TimeFilter>,
	Query(loc_filter): Query<LocationFilter>,
	Query(includes): Query<LocationIncludes>,
	Query(p_opts): Query<PaginationOptions>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let (total, truncated, locations) = Location::search(
		loc_filter,
		time_filter,
		includes,
		p_opts.into(),
		&conn,
	)
	.await?;

	let l_ids = locations.iter().map(|l| l.location.id).collect::<Vec<_>>();

	let (times, tags, imgs) = tokio::join!(
		OpeningTime::get_for_locations(
			l_ids.clone(),
			OpeningTimeIncludes::default(),
			&conn
		),
		Tag::get_for_locations(l_ids.clone(), &conn),
		Image::get_for_locations(l_ids, &conn),
	);

	let times = times?;
	let tags = tags?;
	let imgs = imgs?;

	let locations = Location::group(locations, &times, &tags, &imgs);

	let locations: Result<Vec<LocationResponse>, _> =
		locations.into_iter().map(|l| l.build_response(&config)).collect();
	let locations = locations?;

	let paginated = p_opts.paginate(total, truncated, locations);

	Ok((StatusCode::OK, Json(paginated)))
}

/// Update a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn update_location(
	State(pool): State<DbPool>,
	State(config): State<Config>,
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

	can_manage |= Location::owner_or_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	let loc_update = request.to_insertable(session.data.profile_id);
	let updated_loc = loc_update.apply_to(id, includes, &conn).await?;
	let response: LocationResponse = updated_loc.build_response(&config)?;

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

	can_manage |= Location::admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::ApproveLocation,
		LocationPermissions::ManageLocation,
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

	can_manage |= Location::admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::ApproveLocation,
		LocationPermissions::ManageLocation,
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

#[instrument(skip(pool))]
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

	can_manage |= Location::owner_or_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	Tag::bulk_set(id, data.tags, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

#[instrument]
pub async fn get_all_location_permissions() -> impl IntoResponse {
	let perms = LocationPermissions::names();

	(StatusCode::OK, Json(perms))
}

#[instrument(skip(pool))]
pub async fn get_location_members(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let can_manage = Location::owner_or_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	let members = Location::get_members(id, &conn).await?;
	let response: Vec<ProfilePermissionsResponse> = members
		.into_iter()
		.map(|data| data.build_response(&config))
		.collect::<Result<_, _>>()?;

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn add_location_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path(id): Path<i32>,
	Json(request): Json<CreateLocationMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let can_manage = Location::owner_or_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	let new_loc_profile = request.to_insertable(id, session.data.profile_id);
	let (member, img, perms) = new_loc_profile.insert(&conn).await?;
	let response: ProfilePermissionsResponse =
		(member, img, perms).build_response(&config)?;

	Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(pool))]
pub async fn delete_location_member(
	State(pool): State<DbPool>,
	session: Session,
	Path((l_id, p_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let can_manage = Location::owner_or_admin_or(
		session.data.profile_id,
		l_id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation
			| LocationPermissions::ManageMembers,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	Location::delete_member(l_id, p_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

#[instrument(skip(pool))]
pub async fn update_location_member(
	State(pool): State<DbPool>,
	State(config): State<Config>,
	session: Session,
	Path((l_id, p_id)): Path<(i32, i32)>,
	Json(request): Json<UpdateLocationMemberRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let can_manage = Location::owner_or_admin_or(
		session.data.profile_id,
		l_id,
		AuthorityPermissions::ManageLocation,
		LocationPermissions::ManageLocation
			| LocationPermissions::ManageMembers,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	let loc_update = request.to_insertable(session.data.profile_id);
	let (updated_member, img, perms) =
		loc_update.apply_to(l_id, p_id, &conn).await?;
	let response: ProfilePermissionsResponse =
		(updated_member, img, perms).build_response(&config)?;

	Ok((StatusCode::OK, Json(response)))
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

	can_manage |= Location::owner_or_admin_or(
		session.data.profile_id,
		id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::DeleteLocation,
		LocationPermissions::ManageLocation
			| LocationPermissions::DeleteLocation,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	Location::delete_by_id(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
