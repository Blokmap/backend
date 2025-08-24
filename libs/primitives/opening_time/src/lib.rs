#[macro_use]
extern crate tracing;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use common::{DbConn, Error};
use db::opening_time;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Date;
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
	pub updated_at:       NaiveDateTime,
}

impl PrimitiveOpeningTime {
	/// Get a [`PrimitiveOpeningTime`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(t_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let opening_time = conn
			.interact(move |conn| {
				use self::opening_time::dsl::*;

				opening_time
					.find(t_id)
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		Ok(opening_time)
	}

	/// Get all the [`PrimitiveOpeningTimes`] for a specific location limited
	/// to the current week
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		l_id: i32,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let now = Utc::now().date_naive();
		let week = now.week(chrono::Weekday::Mon);
		let start = week.first_day();
		let end = week.last_day();

		let times = conn
			.interact(move |conn| {
				use self::opening_time::dsl::*;

				opening_time
					.filter(location_id.eq(l_id))
					.filter(start.into_sql::<Date>().le(day))
					.filter(end.into_sql::<Date>().ge(day))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(times)
	}

	/// Get all the [`PrimitiveOpeningTimes`] for a list of locations
	#[instrument(skip(conn))]
	pub async fn get_for_locations(
		l_ids: Vec<i32>,
		conn: &DbConn,
	) -> Result<Vec<(i32, Self)>, Error> {
		let times = conn
			.interact(move |conn| {
				use self::opening_time::dsl::*;

				opening_time
					.filter(location_id.eq_any(l_ids))
					.select((location_id, Self::as_select()))
					.get_results(conn)
			})
			.await??;

		Ok(times)
	}
}
