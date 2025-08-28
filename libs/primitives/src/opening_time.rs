use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use db::opening_time;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone,
	Debug,
	Deserialize,
	Identifiable,
	Queryable,
	QueryableByName,
	Selectable,
	Serialize,
)]
#[diesel(table_name = opening_time)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveOpeningTime {
	pub id:               i32,
	pub location_id:      i32,
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
