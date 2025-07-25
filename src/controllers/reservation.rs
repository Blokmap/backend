use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{NaiveDateTime, NaiveTime, Utc};
use common::{CreateReservationError, DbPool, Error};
use models::{
	AuthorityPermissions,
	Location,
	LocationPermissions,
	NewReservation,
	PrimitiveLocation,
	PrimitiveOpeningTime,
	PrimitiveReservation,
	Reservation,
	ReservationFilter,
	ReservationIncludes,
};

use crate::Session;
use crate::schemas::reservation::{
	CreateReservationRequest,
	ReservationResponse,
};

#[instrument(skip(pool))]
pub async fn get_reservation_for_location(
	State(pool): State<DbPool>,
	session: Session,
	Path(loc_id): Path<i32>,
	Query(filter): Query<ReservationFilter>,
	Query(includes): Query<ReservationIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= Location::admin_or(
		session.data.profile_id,
		loc_id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::ManageReservations,
		LocationPermissions::ManageLocation
			| LocationPermissions::ManageReservations,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	let reservations =
		Reservation::for_location(loc_id, filter, includes, &conn).await?;
	let response: Vec<ReservationResponse> =
		reservations.into_iter().map(Into::into).collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn get_reservation_for_opening_time(
	State(pool): State<DbPool>,
	session: Session,
	Path((l_id, t_id)): Path<(i32, i32)>,
	Query(includes): Query<ReservationIncludes>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= Location::admin_or(
		session.data.profile_id,
		l_id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::ManageReservations,
		LocationPermissions::ManageLocation
			| LocationPermissions::ManageReservations,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	let loc = PrimitiveLocation::get_by_id(l_id, &conn).await?;

	let reservations =
		Reservation::for_opening_time(t_id, includes, &conn).await?;
	let response: Vec<ReservationResponse> = reservations
		.into_iter()
		.map(|(time, res)| From::from((&loc, &time, res)))
		.collect();

	Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(pool))]
pub async fn create_reservation(
	State(pool): State<DbPool>,
	session: Session,
	Path((l_id, t_id)): Path<(i32, i32)>,
	Query(includes): Query<ReservationIncludes>,
	Json(request): Json<CreateReservationRequest>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let time = PrimitiveOpeningTime::get_by_id(t_id, &conn).await?;

	check_reservation_bounds(
		time.start_time,
		time.end_time,
		request.start_time,
		request.end_time,
	)?;

	check_reservation_period(time.reservable_from, time.reservable_until)?;

	let loc = PrimitiveLocation::get_by_id(l_id, &conn).await?;

	let block_size = i64::from(loc.reservation_block_size);

	let offset = (request.start_time - time.start_time).num_minutes();
	#[allow(clippy::cast_possible_truncation)]
	let base_block_index = (offset / block_size) as i32;

	let span = (request.end_time - request.start_time).num_minutes();
	#[allow(clippy::cast_possible_truncation)]
	let block_count = (span / block_size) as i32;

	check_reservation_length(
		loc.min_reservation_length,
		loc.max_reservation_length,
		block_count,
	)?;

	#[allow(clippy::cast_possible_truncation)]
	let num_blocks =
		((time.end_time - time.start_time).num_minutes() / block_size) as i32;
	let spans = Reservation::get_spans_for_opening_time(t_id, &conn).await?;

	check_reservation_occupation(
		num_blocks,
		&spans,
		time.seat_count.unwrap_or(loc.seat_count),
	)?;

	let new_reservation = NewReservation {
		profile_id: session.data.profile_id,
		opening_time_id: t_id,
		base_block_index,
		block_count,
	};

	let new_reservation = new_reservation.insert(includes, &conn).await?;
	let response = ReservationResponse::from((loc, time, new_reservation));

	Ok((StatusCode::CREATED, Json(response)))
}

fn check_reservation_bounds(
	min_start_time: NaiveTime,
	max_end_time: NaiveTime,
	start_time: NaiveTime,
	end_time: NaiveTime,
) -> Result<(), Error> {
	if start_time < min_start_time || end_time > max_end_time {
		return Err(CreateReservationError::OutOfBounds {
			start: min_start_time,
			end:   max_end_time,
		}
		.into());
	}

	Ok(())
}

fn check_reservation_period(
	from: Option<NaiveDateTime>,
	until: Option<NaiveDateTime>,
) -> Result<(), Error> {
	#[allow(clippy::collapsible_if)]
	if let Some(from) = from {
		if Utc::now().naive_utc() < from {
			return Err(CreateReservationError::NotReservableYet(from).into());
		}
	}

	#[allow(clippy::collapsible_if)]
	if let Some(until) = until {
		if Utc::now().naive_utc() > until {
			return Err(
				CreateReservationError::NotReservableAnymore(until).into()
			);
		}
	}

	Ok(())
}

fn check_reservation_length(
	min: Option<i32>,
	max: Option<i32>,
	len: i32,
) -> Result<(), Error> {
	#[allow(clippy::collapsible_if)]
	if let Some(min) = min {
		if len < min {
			return Err(CreateReservationError::ReservationTooShort(min).into());
		}
	}

	#[allow(clippy::collapsible_if)]
	if let Some(max) = max {
		if len > max {
			return Err(CreateReservationError::ReservationTooLong(max).into());
		}
	}

	Ok(())
}

fn check_reservation_occupation(
	blocks: i32,
	spans: &[(i32, i32)],
	seats: i32,
) -> Result<(), Error> {
	let mut occupation = HashMap::<i32, i32>::new();

	for i in 0..blocks {
		let entry = occupation.entry(i).or_insert(0);

		for span in spans {
			if span.0 <= *entry && (span.0 + span.1) >= *entry {
				*entry += 1;
			}
		}
	}

	let mut full = vec![];

	for (block, reservations) in occupation {
		// + 1 because we want to know if adding another reservation will
		// overflow
		if reservations + 1 > seats {
			full.push(block);
		}
	}

	if !full.is_empty() {
		return Err(CreateReservationError::Full(full).into());
	}

	Ok(())
}

#[instrument(skip(pool))]
pub async fn delete_reservation(
	State(pool): State<DbPool>,
	session: Session,
	Path((l_id, t_id, r_id)): Path<(i32, i32, i32)>,
) -> Result<impl IntoResponse, Error> {
	let conn = pool.get().await?;

	let reservation = PrimitiveReservation::get_by_id(r_id, &conn).await?;

	let mut can_manage = false;

	if session.data.profile_is_admin {
		can_manage = true;
	}

	can_manage |= Location::admin_or(
		session.data.profile_id,
		l_id,
		AuthorityPermissions::ManageLocation
			| AuthorityPermissions::ManageReservations,
		LocationPermissions::ManageLocation
			| LocationPermissions::ManageReservations,
		&conn,
	)
	.await?;

	if !can_manage {
		return Err(Error::Forbidden);
	}

	if reservation.profile_id != session.data.profile_id
		&& !session.data.profile_is_admin
	{
		return Err(Error::Forbidden);
	}

	Reservation::delete_by_id(r_id, &conn).await?;

	Ok(StatusCode::NO_CONTENT)
}
