use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Date};
use serde::{Deserialize, Serialize};

use crate::SimpleProfile;
use crate::schema::{creator, opening_time, simple_profile, updater};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTimeFilter {
	pub start_date: Option<NaiveDate>,
	pub end_date:   Option<NaiveDate>,
}

type BoxedCondition<S, T> = Box<dyn BoxableExpression<S, Pg, SqlType = T>>;

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
	pub opening_time: PrimitiveOpeningTime,
	pub created_by:   Option<Option<SimpleProfile>>,
	pub updated_by:   Option<Option<SimpleProfile>>,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
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

impl OpeningTime {
	/// Get an [`OpeningTime`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		t_id: i32,
		includes: OpeningTimeIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let time: (
			PrimitiveOpeningTime,
			Option<SimpleProfile>,
			Option<SimpleProfile>,
		) = conn
			.interact(move |conn| {
				use self::opening_time::dsl::*;

				opening_time
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							created_by.eq(
								creator.field(simple_profile::id).nullable(),
							),
						),
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							updated_by.eq(
								updater.field(simple_profile::id).nullable(),
							),
						),
					))
					.filter(id.eq(t_id))
					.select((
						PrimitiveOpeningTime::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let time = OpeningTime {
			opening_time: time.0,
			created_by:   if includes.created_by { Some(time.1) } else { None },
			updated_by:   if includes.updated_by { Some(time.2) } else { None },
		};

		Ok(time)
	}

	/// Get all the [`OpeningTimes`] for a specific location
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		loc_id: i32,
		time_filter: OpeningTimeFilter,
		includes: OpeningTimeIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let start_filter: BoxedCondition<_, Bool> =
			if let Some(start_date) = time_filter.start_date {
				Box::new(start_date.into_sql::<Date>().ge(opening_time::day))
			} else {
				Box::new(true.into_sql::<Bool>().eq(true))
			};

		let end_filter: BoxedCondition<_, Bool> =
			if let Some(end_date) = time_filter.end_date {
				Box::new(end_date.into_sql::<Date>().le(opening_time::day))
			} else {
				Box::new(true.into_sql::<Bool>().eq(true))
			};

		let times = conn
			.interact(move |conn| {
				use self::opening_time::dsl::*;

				opening_time
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							created_by.eq(
								creator.field(simple_profile::id).nullable(),
							),
						),
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							updated_by.eq(
								updater.field(simple_profile::id).nullable(),
							),
						),
					))
					.filter(location_id.eq(loc_id))
					.filter(start_filter)
					.filter(end_filter)
					.select((
						PrimitiveOpeningTime::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(time, cr, up)| {
				OpeningTime {
					opening_time: time,
					created_by:   if includes.created_by {
						Some(cr)
					} else {
						None
					},
					updated_by:   if includes.updated_by {
						Some(up)
					} else {
						None
					},
				}
			})
			.collect();

		Ok(times)
	}

	/// Delete an [`OpeningTime`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(t_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::schema::opening_time::dsl::*;

			diesel::delete(opening_time.find(t_id)).execute(conn)
		})
		.await??;

		info!("deleted opening_time with id {t_id}");

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
	/// Insert this [`NewOpeningTime`] into the database.
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		includes: OpeningTimeIncludes,
		conn: &DbConn,
	) -> Result<OpeningTime, Error> {
		let time = conn
			.interact(|conn| {
				use self::opening_time::dsl::*;

				diesel::insert_into(opening_time)
					.values(self)
					.returning(PrimitiveOpeningTime::as_returning())
					.get_result(conn)
			})
			.await??;

		let time = OpeningTime::get_by_id(time.id, includes, conn).await?;

		info!("created opening_time {time:?}");

		Ok(time)
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
			use crate::schema::opening_time::dsl::*;

			diesel::update(opening_time.find(t_id)).set(self).execute(conn)
		})
		.await??;

		let time = OpeningTime::get_by_id(t_id, includes, conn).await?;

		info!("updated opening_time {time:?}");

		Ok(time)
	}
}
