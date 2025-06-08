use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use models::OpeningTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTimeResponse {
	pub id:               i32,
	pub day:              NaiveDate,
	pub start_time:       NaiveTime,
	pub end_time:         NaiveTime,
	pub seat_count:       Option<i32>,
	pub reservable_from:  Option<NaiveDateTime>,
	pub reservable_until: Option<NaiveDateTime>,
	pub created_at:       NaiveDateTime,
	pub created_by:       Option<i32>,
	pub updated_at:       NaiveDateTime,
	pub updated_by:       Option<i32>,
}

impl From<OpeningTime> for OpeningTimeResponse {
	fn from(value: OpeningTime) -> Self {
		Self {
			id:               value.id,
			day:              value.day,
			start_time:       value.start_time,
			end_time:         value.end_time,
			seat_count:       value.seat_count,
			reservable_from:  value.reservable_from,
			reservable_until: value.reservable_until,
			created_at:       value.created_at,
			created_by:       value.created_by,
			updated_at:       value.updated_at,
			updated_by:       value.updated_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateOpeningTimeRequest {
	pub day:              NaiveDate,
	pub start_time:       NaiveTime,
	pub end_time:         NaiveTime,
	pub seat_count:       Option<i32>,
	pub reservable_from:  Option<NaiveDateTime>,
	pub reservable_until: Option<NaiveDateTime>,
}
