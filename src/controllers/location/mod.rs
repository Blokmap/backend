//! Controllers for [`Location`]s

use ::image::{Image, ImageIncludes};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, NoContent};
use common::{DbPool, Error};
use location::{Location, LocationFilter, LocationIncludes, Point};
use opening_time::{OpeningTime, OpeningTimeIncludes, TimeFilter};
use permissions::Permissions;
use tag::{Tag, TagIncludes};
use validator::Validate;

use crate::schemas::BuildResponse;
use crate::schemas::location::{
	CreateLocationRequest,
	LocationResponse,
	NearestLocationResponse,
	RejectLocationRequest,
	UpdateLocationRequest,
};
use crate::schemas::pagination::PaginationOptions;
use crate::schemas::tag::SetLocationTagsRequest;
use crate::{Config, Session};

mod image;
mod member;
mod role;

pub(crate) use image::*;
pub(crate) use member::*;
pub(crate) use role::*;

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
	let response = records.build_response(includes, &config)?;

	Ok((StatusCode::CREATED, Json(response)))
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
	let response = result.build_response(includes, &config)?;

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

	let l_ids = locations.iter().map(|l| l.primitive.id).collect::<Vec<_>>();

	let (times, tags, imgs) = tokio::join!(
		OpeningTime::get_for_locations(
			l_ids.clone(),
			OpeningTimeIncludes::default(),
			&conn
		),
		Tag::get_for_locations(l_ids.clone(), TagIncludes::default(), &conn),
		Image::get_for_locations(l_ids, ImageIncludes::default(), &conn),
	);

	let times = times?;
	let tags = tags?;
	let imgs = imgs?;

	let locations = Location::group(locations, &times, &tags, &imgs);

	let locations: Vec<LocationResponse> = locations
		.into_iter()
		.map(|l| l.build_response(includes, &config))
		.collect::<Result<_, _>>()?;

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
	Permissions::check_for_location(
		id,
		session.data.profile_id,
		Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	let loc_update = request.to_insertable(session.data.profile_id);
	let updated_loc = loc_update.apply_to(id, includes, &conn).await?;
	let response = updated_loc.build_response(includes, &config)?;

	Ok((StatusCode::OK, Json(response)))
}

/// Approve a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn approve_location(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let location =
		Location::get_by_id(id, LocationIncludes::default(), &conn).await?;
	let location = location.0.primitive;

	if let Some(auth_id) = location.authority_id {
		Permissions::check_for_authority(
			auth_id,
			session.data.profile_id,
			Permissions::AuthApproveLocations
				| Permissions::AuthAdministrator
				| Permissions::InstAdministrator,
			&pool,
		)
		.await?;
	} else if !session.data.is_admin {
		return Err(Error::Forbidden);
	}

	Location::approve_by(id, session.data.profile_id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

/// Reject a location in the database.
#[instrument(skip(pool))]
pub(crate) async fn reject_location(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Json(request): Json<RejectLocationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let location =
		Location::get_by_id(id, LocationIncludes::default(), &conn).await?;
	let location = location.0.primitive;

	if let Some(auth_id) = location.authority_id {
		Permissions::check_for_authority(
			auth_id,
			session.data.profile_id,
			Permissions::AuthApproveLocations
				| Permissions::AuthAdministrator
				| Permissions::InstAdministrator,
			&pool,
		)
		.await?;
	} else if !session.data.is_admin {
		return Err(Error::Forbidden);
	}

	Location::reject_by(id, session.data.profile_id, request.reason, &conn)
		.await?;

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

	let location =
		Location::get_simple_by_id(id, LocationIncludes::default(), &conn)
			.await?;
	let location = location.primitive;

	if let Some(auth_id) = location.authority_id {
		Permissions::check_for_authority(
			auth_id,
			session.data.profile_id,
			Permissions::AuthDeleteLocations
				| Permissions::AuthAdministrator
				| Permissions::InstAdministrator,
			&pool,
		)
		.await?;
	} else if location.created_by != Some(session.data.profile_id) {
		Permissions::check_for_location(
			id,
			session.data.profile_id,
			Permissions::LocAdministrator,
			&pool,
		)
		.await?;
	}

	Location::delete_by_id(id, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}

#[instrument(skip(pool))]
pub async fn set_location_tags(
	State(pool): State<DbPool>,
	session: Session,
	Path(id): Path<i32>,
	Json(data): Json<SetLocationTagsRequest>,
) -> Result<impl IntoResponse, Error> {
	Permissions::check_for_location(
		id,
		session.data.profile_id,
		Permissions::LocAdministrator
			| Permissions::AuthAdministrator
			| Permissions::InstAdministrator,
		&pool,
	)
	.await?;

	let conn = pool.get().await?;

	Tag::bulk_set(id, data.tags, &conn).await?;

	Ok((StatusCode::NO_CONTENT, NoContent))
}
