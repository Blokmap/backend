use std::borrow::Borrow;

use chrono::{Duration, NaiveDateTime, NaiveTime};
use models::{
	PrimitiveLocation,
	PrimitiveOpeningTime,
	Reservation,
	SimpleProfile,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationResponse {
	pub id:               i32,
	pub opening_time_id:  i32,
	pub base_block_index: i32,
	pub block_count:      i32,
	pub start_time:       NaiveDateTime,
	pub end_time:         NaiveDateTime,
	pub created_at:       NaiveDateTime,
	pub updated_at:       NaiveDateTime,
	pub confirmed_at:     Option<NaiveDateTime>,
	pub confirmed_by:     Option<Option<SimpleProfile>>,
	pub location:         PrimitiveLocation,
}

impl<L, T> From<(L, T, Reservation)> for ReservationResponse
where
	L: Borrow<PrimitiveLocation>,
	T: Borrow<PrimitiveOpeningTime>,
{
	fn from(value: (L, T, Reservation)) -> Self {
		let block_size = value.0.borrow().reservation_block_size;
		let block_day = value.1.borrow().day;
		let block_start_time = value.1.borrow().start_time;

		let base_idx = value.2.reservation.base_block_index;
		let block_count = value.2.reservation.block_count;

		let start_time = block_day.and_time(
			block_start_time
				+ Duration::minutes((base_idx * block_size).into()),
		);
		let end_time =
			start_time + Duration::minutes((block_count * block_size).into());

		Self {
			id: value.2.reservation.id,
			opening_time_id: value.2.reservation.opening_time_id,
			base_block_index: value.2.reservation.base_block_index,
			block_count: value.2.reservation.block_count,
			start_time,
			end_time,
			created_at: value.2.reservation.created_at,
			updated_at: value.2.reservation.updated_at,
			confirmed_at: value.2.reservation.confirmed_at,
			confirmed_by: value.2.confirmed_by,
			location: value.0.borrow().clone(),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReservationRequest {
	pub start_time: NaiveTime,
	pub end_time:   NaiveTime,
}
