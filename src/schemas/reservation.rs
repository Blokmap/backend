use chrono::{Duration, NaiveDateTime, NaiveTime};
use models::{RESERVATION_BLOCK_SIZE_MINUTES, Reservation, ReservationState};
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
	pub created_by:       Option<ProfileResponse>,
	pub updated_at:       NaiveDateTime,
	pub confirmed_at:     Option<NaiveDateTime>,
	#[serde(serialize_with = "ser_includes")]
	pub confirmed_by:     Option<Option<ProfileResponse>>,

	pub opening_time: OpeningTimeResponse,
	pub location:     LocationResponse,
}

impl From<Reservation> for ReservationResponse {
	fn from(value: Reservation) -> Self {
		let location = value.location;
		let opening_time = value.opening_time;

		let reservation = value.reservation;

		let block_day = opening_time.day;
		let block_start_time = opening_time.start_time;

		let base_idx = reservation.base_block_index;
		let block_count = reservation.block_count;

		let start_offset = Duration::minutes(
			(base_idx * RESERVATION_BLOCK_SIZE_MINUTES).into(),
		);
		let start_time = block_day.and_time(block_start_time + start_offset);

		let end_offset = Duration::minutes(
			((base_idx + block_count) * RESERVATION_BLOCK_SIZE_MINUTES).into(),
		);
		let end_time = start_time + end_offset;

		Self {
			id: reservation.id,
			state: reservation.state,
			opening_time_id: reservation.opening_time_id,
			base_block_index: reservation.base_block_index,
			block_count: reservation.block_count,
			created_at: reservation.created_at,
			created_by: value.profile.map(Into::into),
			updated_at: reservation.updated_at,
			confirmed_at: reservation.confirmed_at,
			confirmed_by: value.confirmed_by.map(|p| p.map(Into::into)),
			opening_time: opening_time.into(),
			location: location.into(),
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
