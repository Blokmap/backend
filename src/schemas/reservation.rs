use std::borrow::Borrow;

use chrono::{Duration, NaiveDateTime, NaiveTime};
use models::{
	PrimitiveLocation,
	PrimitiveOpeningTime,
	Reservation,
	ReservationState,
};
use serde::{Deserialize, Serialize};

use crate::schemas::location::LocationResponse;
use crate::schemas::opening_time::OpeningTimeResponse;
use crate::schemas::profile::ProfileResponse;
use crate::schemas::ser_includes;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationResponse {
	pub id:               i32,
	pub state:            ReservationState,
	pub opening_time_id:  i32,
	pub base_block_index: i32,
	pub block_count:      i32,
	pub start_time:       NaiveDateTime,
	pub end_time:         NaiveDateTime,
	pub created_at:       NaiveDateTime,
	pub updated_at:       NaiveDateTime,
	pub confirmed_at:     Option<NaiveDateTime>,
	#[serde(serialize_with = "ser_includes")]
	pub confirmed_by:     Option<Option<ProfileResponse>>,
	#[serde(serialize_with = "ser_includes")]
	pub opening_time:     Option<Option<OpeningTimeResponse>>,
	#[serde(serialize_with = "ser_includes")]
	pub location:         Option<Option<LocationResponse>>,
}

impl<L, T> From<(L, T, Reservation)> for ReservationResponse
where
	L: Borrow<PrimitiveLocation>,
	T: Borrow<PrimitiveOpeningTime>,
{
	fn from(value: (L, T, Reservation)) -> Self {
		let location = value.0.borrow();
		let opening_time = value.1.borrow();

		let relations = value.2;
		let reservation = relations.reservation;

		let block_size = location.reservation_block_size;
		let block_day = opening_time.day;
		let block_start_time = opening_time.start_time;

		let base_idx = reservation.base_block_index;
		let block_count = reservation.block_count;

		let start_offset = Duration::minutes((base_idx * block_size).into());
		let start_time = block_day.and_time(block_start_time + start_offset);

		let end_offset =
			Duration::minutes(((base_idx + block_count) * block_size).into());
		let end_time = start_time + end_offset;

		Self {
			id: reservation.id,
			state: reservation.state,
			opening_time_id: reservation.opening_time_id,
			base_block_index: reservation.base_block_index,
			block_count: reservation.block_count,
			created_at: reservation.created_at,
			updated_at: reservation.updated_at,
			confirmed_at: reservation.confirmed_at,
			confirmed_by: relations.confirmed_by.map(|p| p.map(Into::into)),
			opening_time: relations.opening_time.map(|p| p.map(Into::into)),
			location: relations.location.map(|p| p.map(Into::into)),
			start_time,
			end_time,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReservationRequest {
	pub start_time: NaiveTime,
	pub end_time:   NaiveTime,
}
