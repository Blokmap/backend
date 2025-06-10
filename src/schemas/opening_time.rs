use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use models::{NewOpeningTime, OpeningTime, OpeningTimeUpdate, SimpleProfile};
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
	pub created_by:       Option<Option<SimpleProfile>>,
	pub updated_at:       NaiveDateTime,
	pub updated_by:       Option<Option<SimpleProfile>>,
}

impl From<OpeningTime> for OpeningTimeResponse {
	fn from(value: OpeningTime) -> Self {
		Self {
			id:               value.opening_time.id,
			day:              value.opening_time.day,
			start_time:       value.opening_time.start_time,
			end_time:         value.opening_time.end_time,
			seat_count:       value.opening_time.seat_count,
			reservable_from:  value.opening_time.reservable_from,
			reservable_until: value.opening_time.reservable_until,
			created_at:       value.opening_time.created_at,
			created_by:       value.created_by,
			updated_at:       value.opening_time.updated_at,
			updated_by:       value.updated_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOpeningTimeRequest {
	pub day:              NaiveDate,
	pub start_time:       NaiveTime,
	pub end_time:         NaiveTime,
	pub seat_count:       Option<i32>,
	pub reservable_from:  Option<NaiveDateTime>,
	pub reservable_until: Option<NaiveDateTime>,
}

impl CreateOpeningTimeRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		location_id: i32,
		created_by: i32,
	) -> NewOpeningTime {
		NewOpeningTime {
			location_id,
			day: self.day,
			start_time: self.start_time,
			end_time: self.end_time,
			seat_count: self.seat_count,
			reservable_from: self.reservable_from,
			reservable_until: self.reservable_until,
			created_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOpeningTimeRequest {
	pub day:              Option<NaiveDate>,
	pub start_time:       Option<NaiveTime>,
	pub end_time:         Option<NaiveTime>,
	pub seat_count:       Option<i32>,
	pub reservable_from:  Option<NaiveDateTime>,
	pub reservable_until: Option<NaiveDateTime>,
}

impl UpdateOpeningTimeRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> OpeningTimeUpdate {
		OpeningTimeUpdate {
			day: self.day,
			start_time: self.start_time,
			end_time: self.end_time,
			seat_count: self.seat_count,
			reservable_from: self.reservable_from,
			reservable_until: self.reservable_until,
			updated_by,
		}
	}
}
