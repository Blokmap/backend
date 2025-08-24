use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Date, Time};
use serde::{Deserialize, Serialize};

use crate::db::{creator, opening_time, profile, updater};
use crate::{BoxedCondition, PrimitiveProfile, ToFilter};

pub type JoinedOpeningTimeData =
	(PrimitiveOpeningTime, Option<PrimitiveProfile>, Option<PrimitiveProfile>);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeBoundsFilter {
	pub start_date: Option<NaiveDate>,
	pub end_date:   Option<NaiveDate>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeFilter {
	pub open_on_day:  Option<NaiveDate>,
	pub open_on_time: Option<NaiveTime>,
}

impl<S> ToFilter<S> for TimeBoundsFilter
where
	S: 'static,
	opening_time::day: SelectableExpression<S>,
{
	type SqlType = Bool;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		let start_filter: BoxedCondition<_, Bool> =
			if let Some(start_date) = self.start_date {
				Box::new(start_date.into_sql::<Date>().le(opening_time::day))
			} else {
				Box::new(true.into_sql::<Bool>().eq(true))
			};

		let end_filter: BoxedCondition<_, Bool> =
			if let Some(end_date) = self.end_date {
				Box::new(end_date.into_sql::<Date>().ge(opening_time::day))
			} else {
				Box::new(true.into_sql::<Bool>().eq(true))
			};

		let filter: BoxedCondition<S, Self::SqlType> =
			Box::new(start_filter.and(end_filter));

		filter
	}
}

impl<S> ToFilter<S> for TimeFilter
where
	S: 'static,
	opening_time::id: SelectableExpression<S>,
	opening_time::day: SelectableExpression<S>,
	opening_time::start_time: SelectableExpression<S>,
	opening_time::end_time: SelectableExpression<S>,
{
	type SqlType = Bool;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		let mut filter: BoxedCondition<S, Self::SqlType> =
			if let Some(open_on_day) = self.open_on_day {
				Box::new(open_on_day.into_sql::<Date>().eq(opening_time::day))
			} else {
				Box::new(true.into_sql::<Bool>())
			};

		if let Some(open_on_time) = self.open_on_time {
			filter =
				Box::new(filter.and(open_on_time.into_sql::<Time>().between(
					opening_time::start_time,
					opening_time::end_time,
				)));
		}

		filter
	}
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct OpeningTimeIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = opening_time)]
#[diesel(check_for_backend(Pg))]
pub struct OpeningTime {
	pub opening_time:   PrimitiveOpeningTime,
	pub seat_occupancy: Option<i32>,
	pub created_by:     Option<Option<PrimitiveProfile>>,
	pub updated_by:     Option<Option<PrimitiveProfile>>,
}

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
				use crate::db::opening_time::dsl::*;

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

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl OpeningTime {
	/// Build a query with all required (dynamic) joins to select a full
	/// location data tuple
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn joined_query(includes: OpeningTimeIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		opening_time::table
			.left_outer_join(
				creator.on(inc_created_by.into_sql::<Bool>().and(
					opening_time::created_by
						.eq(creator.field(profile::id).nullable()),
				)),
			)
			.left_outer_join(
				updater.on(inc_updated_by.into_sql::<Bool>().and(
					opening_time::updated_by
						.eq(updater.field(profile::id).nullable()),
				)),
			)
	}

	/// Construct a full [`OpeningTime`] struct from the data returned by a
	/// joined query
	fn from_joined(
		includes: OpeningTimeIncludes,
		data: JoinedOpeningTimeData,
	) -> Self {
		Self {
			opening_time:   data.0,
			seat_occupancy: None,
			created_by:     if includes.created_by {
				Some(data.1)
			} else {
				None
			},
			updated_by:     if includes.updated_by {
				Some(data.2)
			} else {
				None
			},
		}
	}

	/// Get an [`OpeningTime`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		t_id: i32,
		includes: OpeningTimeIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query(includes);

		let time = conn
			.interact(move |conn| {
				query
					.filter(opening_time::id.eq(t_id))
					.select((
						PrimitiveOpeningTime::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let time = Self::from_joined(includes, time);

		Ok(time)
	}

	/// Get all the [`OpeningTimes`] for a specific location
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		loc_id: i32,
		time_filter: TimeBoundsFilter,
		includes: OpeningTimeIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let filter = time_filter.to_filter();
		let query = Self::joined_query(includes);

		let times = conn
			.interact(move |conn| {
				use self::opening_time::dsl::*;

				query
					.filter(location_id.eq(loc_id))
					.filter(filter)
					.select((
						PrimitiveOpeningTime::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		Ok(times)
	}

	/// Search through all [`OpeningTime`]s
	#[instrument(skip(conn))]
	pub async fn search(
		time_filter: TimeFilter,
		conn: &DbConn,
	) -> Result<Vec<PrimitiveOpeningTime>, Error> {
		let filter = time_filter.to_filter();

		let bounds_filter = if let Some(open_on_day) = time_filter.open_on_day {
			let week = open_on_day.week(Weekday::Mon);
			// I don't think blokmap will still be used in 264.000 AD so unwrap
			// should be safe
			let week_start = week.checked_first_day().unwrap();
			let week_end = week.checked_last_day().unwrap();

			let bounds_filter = TimeBoundsFilter {
				start_date: Some(week_start),
				end_date:   Some(week_end),
			};

			bounds_filter.to_filter()
		} else {
			let now = Utc::now().date_naive();
			let week = now.week(Weekday::Mon);
			let week_start = week.checked_first_day().unwrap();
			let week_end = week.checked_last_day().unwrap();

			let bounds_filter = TimeBoundsFilter {
				start_date: Some(week_start),
				end_date:   Some(week_end),
			};

			bounds_filter.to_filter()
		};

		let filter = Box::new(filter.and(bounds_filter));

		let times = conn
			.interact(move |conn| {
				use crate::db::opening_time::dsl::*;

				opening_time
					.filter(filter)
					.select(PrimitiveOpeningTime::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(times)
	}

	/// Delete an [`OpeningTime`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(t_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::db::opening_time::dsl::*;

			diesel::delete(opening_time.find(t_id)).execute(conn)
		})
		.await??;

		info!("deleted opening_time with id {t_id}");

		Ok(())
	}

	/// Delete all [`OpeningTime`]s for a specific location
	#[instrument(skip(conn))]
	pub async fn delete_by_location_id(
		loc_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::db::opening_time::dsl::*;

			diesel::delete(opening_time.filter(location_id.eq(loc_id)))
				.execute(conn)
		})
		.await??;

		info!("deleted opening_times for location with id {loc_id}");

		Ok(())
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = opening_time)]
#[diesel(check_for_backend(Pg))]
pub struct NewOpeningTime {
	pub location_id:      i32,
	pub day:              NaiveDate,
	pub start_time:       NaiveTime,
	pub end_time:         NaiveTime,
	pub seat_count:       Option<i32>,
	pub reservable_from:  Option<NaiveDateTime>,
	pub reservable_until: Option<NaiveDateTime>,
	pub created_by:       i32,
}

impl NewOpeningTime {
	/// Insert a list of [`NewOpeningTime`] into the database.
	#[instrument(skip(conn))]
	pub async fn bulk_insert(
		times: Vec<Self>,
		includes: OpeningTimeIncludes,
		conn: &DbConn,
	) -> Result<Vec<PrimitiveOpeningTime>, Error> {
		let times = conn
			.interact(|conn| {
				use self::opening_time::dsl::*;

				diesel::insert_into(opening_time)
					.values(times)
					.returning(PrimitiveOpeningTime::as_returning())
					.get_results(conn)
			})
			.await??;

		Ok(times)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize, Serialize)]
#[diesel(table_name = opening_time)]
#[diesel(check_for_backend(Pg))]
pub struct OpeningTimeUpdate {
	pub day:              Option<NaiveDate>,
	pub start_time:       Option<NaiveTime>,
	pub end_time:         Option<NaiveTime>,
	pub seat_count:       Option<i32>,
	pub reservable_from:  Option<NaiveDateTime>,
	pub reservable_until: Option<NaiveDateTime>,
	pub updated_by:       i32,
}

impl OpeningTimeUpdate {
	/// Apply this update to the [`OpeningTime`] with the given id
	#[instrument(skip(conn))]
	pub async fn apply_to(
		self,
		t_id: i32,
		includes: OpeningTimeIncludes,
		conn: &DbConn,
	) -> Result<OpeningTime, Error> {
		conn.interact(move |conn| {
			use crate::db::opening_time::dsl::*;

			diesel::update(opening_time.find(t_id)).set(self).execute(conn)
		})
		.await??;

		let time = OpeningTime::get_by_id(t_id, includes, conn).await?;

		info!("updated opening_time {time:?}");

		Ok(time)
	}
}
