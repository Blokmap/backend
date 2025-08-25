use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use opening_time::{NewOpeningTime, OpeningTime, OpeningTimeUpdate};
use primitive_opening_time::PrimitiveOpeningTime;
use serde::{Deserialize, Serialize};

use crate::schemas::profile::ProfileResponse;
use crate::schemas::ser_includes;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTimeResponse {
	pub id:               i32,
	pub day:              NaiveDate,
	pub start_time:       NaiveTime,
	pub end_time:         NaiveTime,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub seat_occupancy:   Option<i32>,
	pub seat_count:       Option<i32>,
	pub reservable_from:  Option<NaiveDateTime>,
	pub reservable_until: Option<NaiveDateTime>,
	pub created_at:       NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub created_by:       Option<Option<ProfileResponse>>,
	pub updated_at:       NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub updated_by:       Option<Option<ProfileResponse>>,
}

impl From<OpeningTime> for OpeningTimeResponse {
	fn from(value: OpeningTime) -> Self {
		Self {
			id:               value.opening_time.id,
			day:              value.opening_time.day,
			start_time:       value.opening_time.start_time,
			end_time:         value.opening_time.end_time,
			seat_occupancy:   value.seat_occupancy,
			seat_count:       value.opening_time.seat_count,
			reservable_from:  value.opening_time.reservable_from,
			reservable_until: value.opening_time.reservable_until,
			created_at:       value.opening_time.created_at,
			created_by:       value.created_by.map(|p| p.map(Into::into)),
			updated_at:       value.opening_time.updated_at,
			updated_by:       value.updated_by.map(|p| p.map(Into::into)),
		}
	}
}

impl From<PrimitiveOpeningTime> for OpeningTimeResponse {
	fn from(value: PrimitiveOpeningTime) -> Self {
		Self {
			id:               value.id,
			seat_occupancy:   None,
			day:              value.day,
			start_time:       value.start_time,
			end_time:         value.end_time,
			seat_count:       value.seat_count,
			reservable_from:  value.reservable_from,
			reservable_until: value.reservable_until,
			created_at:       value.created_at,
			created_by:       None,
			updated_at:       value.updated_at,
			updated_by:       None,
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
