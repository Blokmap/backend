use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::OpeningTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTimeResponse {
	pub start_time:    NaiveDateTime,
	pub end_time:      NaiveDateTime,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
	pub created_at:    NaiveDateTime,
	pub updated_at:    NaiveDateTime,
}

impl From<OpeningTime> for OpeningTimeResponse {
	fn from(value: OpeningTime) -> Self {
		Self {
			start_time:    value.start_time,
			end_time:      value.end_time,
			seat_count:    value.seat_count,
			is_reservable: value.is_reservable,
			created_at:    value.created_at,
			updated_at:    value.updated_at,
		}
	}
}
