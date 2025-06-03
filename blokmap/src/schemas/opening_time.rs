use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::models::OpeningTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTimeResponse {
	pub day:             NaiveDate,
	pub start_time:      NaiveTime,
	pub end_time:        NaiveTime,
	pub seat_count:      Option<i32>,
	pub reservable_from: Option<NaiveDateTime>,
	pub created_at:      NaiveDateTime,
	pub updated_at:      NaiveDateTime,
}

impl From<OpeningTime> for OpeningTimeResponse {
	fn from(value: OpeningTime) -> Self {
		Self {
			day:             value.day,
			start_time:      value.start_time,
			end_time:        value.end_time,
			seat_count:      value.seat_count,
			reservable_from: value.reservable_from,
			created_at:      value.created_at,
			updated_at:      value.updated_at,
		}
	}
}
