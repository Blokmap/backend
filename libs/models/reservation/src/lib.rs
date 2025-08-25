#[macro_use]
extern crate tracing;

use chrono::NaiveDate;
use common::{DbConn, Error};
use db::{confirmer, creator, location, opening_time, profile, reservation};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Date};
use models_common::{BoxedCondition, ToFilter};
use primitive_location::PrimitiveLocation;
use primitive_opening_time::PrimitiveOpeningTime;
use primitive_profile::PrimitiveProfile;
use primitive_reservation::PrimitiveReservation;
use serde::{Deserialize, Serialize};

pub type JoinedReservationData = (
	PrimitiveReservation,
	PrimitiveOpeningTime,
	PrimitiveLocation,
	Option<PrimitiveProfile>,
	Option<PrimitiveProfile>,
);

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationFilter {
	pub date:       Option<NaiveDate>,
	pub in_week_of: Option<NaiveDate>,
}

impl<S> ToFilter<S> for ReservationFilter
where
	S: 'static,
	opening_time::day: SelectableExpression<S>,
{
	type SqlType = Bool;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		let mut filter: BoxedCondition<S, Self::SqlType> =
			Box::new(true.into_sql::<Bool>());

		if let Some(date) = self.date {
			filter = Box::new(
				filter.and(date.into_sql::<Date>().eq(opening_time::day)),
			);
		}

		if let Some(in_week_of) = self.in_week_of {
			let week = in_week_of.week(chrono::Weekday::Mon);
			let week_start = week.first_day();
			let week_end = week.last_day();

			filter = Box::new(
				filter.and(opening_time::day.between(week_start, week_end)),
			);
		}

		filter
	}
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(rename_all = "camelCase")]
pub struct ReservationIncludes {
	#[serde(default)]
	pub profile:      bool,
	#[serde(default)]
	pub confirmed_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(Pg))]
pub struct Reservation {
	pub reservation:  PrimitiveReservation,
	pub opening_time: PrimitiveOpeningTime,
	pub location:     PrimitiveLocation,
	pub profile:      Option<PrimitiveProfile>,
	pub confirmed_by: Option<Option<PrimitiveProfile>>,
}

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl Reservation {
	/// Build a query with all required (dynamic) joins to select a full
	/// reservation data tuple
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn joined_query(includes: ReservationIncludes) -> _ {
		let inc_profile: bool = includes.profile;
		let inc_confirmed: bool = includes.confirmed_by;

		let opening_time_join = opening_time::table
			.on(reservation::opening_time_id.eq(opening_time::id));

		let location_join =
			location::table.on(opening_time::location_id.eq(location::id));

		let profile_join = creator.on(reservation::profile_id
			.eq(creator.field(profile::id))
			.and(inc_profile.into_sql::<Bool>()));

		let confirmed_join = confirmer.on(reservation::confirmed_by
			.eq(confirmer.field(profile::id).nullable())
			.and(inc_confirmed.into_sql::<Bool>()));

		reservation::table
			.inner_join(opening_time_join)
			.inner_join(location_join)
			.left_outer_join(profile_join)
			.left_outer_join(confirmed_join)
	}

	/// Construct a full [`Reservation`] struct from the data returned by a
	/// joined query
	fn from_joined(
		includes: ReservationIncludes,
		data: JoinedReservationData,
	) -> Self {
		Self {
			reservation:  data.0,
			opening_time: data.1,
			location:     data.2,
			profile:      if includes.profile { data.3 } else { None },
			confirmed_by: if includes.confirmed_by {
				Some(data.4)
			} else {
				None
			},
		}
	}

	/// Get a [`Reservation`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		r_id: i32,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query(includes);

		let reservation = conn
			.interact(move |conn| {
				use self::reservation::dsl::*;

				query
					.filter(id.eq(r_id))
					.select((
						PrimitiveReservation::as_select(),
						PrimitiveOpeningTime::as_select(),
						PrimitiveLocation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						confirmer.fields(profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let reservation = Self::from_joined(includes, reservation);

		Ok(reservation)
	}

	/// Get all the reservations for a specific [`Location`](crate::Location)
	#[instrument(skip(conn))]
	pub async fn for_location(
		loc_id: i32,
		filter: ReservationFilter,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let filter = filter.to_filter();
		let query = Self::joined_query(includes);

		let reservations = conn
			.interact(move |conn| {
				query
					.filter(location::id.eq(loc_id))
					// .filter(date_filter)
					.filter(filter)
					.select((
						PrimitiveReservation::as_select(),
						PrimitiveOpeningTime::as_select(),
						PrimitiveLocation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						confirmer.fields(profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		Ok(reservations)
	}

	/// Get all the reservations for a specific
	/// [`OpeningTime`](crate::OpeningTime)
	#[instrument(skip(conn))]
	pub async fn for_opening_time(
		t_id: i32,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::joined_query(includes);

		let reservations = conn
			.interact(move |conn| {
				query
					.filter(opening_time::id.eq(t_id))
					.select((
						PrimitiveReservation::as_select(),
						PrimitiveOpeningTime::as_select(),
						PrimitiveLocation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						confirmer.fields(profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		Ok(reservations)
	}

	/// Get all the reservations for a specific [`Profile`](crate::Profile)
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		filter: ReservationFilter,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let filter = filter.to_filter();
		let query = Self::joined_query(includes);

		let reservations = conn
			.interact(move |conn| {
				query
					.filter(reservation::profile_id.eq(p_id))
					.filter(filter)
					.select((
						PrimitiveReservation::as_select(),
						PrimitiveOpeningTime::as_select(),
						PrimitiveLocation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						confirmer.fields(profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??;

		let result = reservations
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		Ok(result)
	}

	/// Get all the block (base, count) pairs a given opening time
	#[instrument(skip(conn))]
	pub async fn get_spans_for_opening_time(
		t_id: i32,
		conn: &DbConn,
	) -> Result<Vec<(i32, i32)>, Error> {
		let pairs = conn
			.interact(move |conn| {
				use self::reservation::dsl::*;

				opening_time::table
					.inner_join(
						reservation.on(opening_time_id.eq(opening_time::id)),
					)
					.filter(opening_time::id.eq(t_id))
					.select((base_block_index, block_count))
					.get_results(conn)
			})
			.await??;

		Ok(pairs)
	}

	/// Delete a [`Reservation`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(r_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::reservation::dsl::*;

			diesel::delete(reservation.find(r_id)).execute(conn)
		})
		.await??;

		info!("deleted reservation with id {r_id}");

		Ok(())
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(Pg))]
pub struct NewReservation {
	pub profile_id:       i32,
	pub opening_time_id:  i32,
	pub base_block_index: i32,
	pub block_count:      i32,
}

impl NewReservation {
	/// Insert this [`NewReservation`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Reservation, Error> {
		let reservation = conn
			.interact(|conn| {
				use self::reservation::dsl::*;

				diesel::insert_into(reservation)
					.values(self)
					.returning(PrimitiveReservation::as_returning())
					.get_result(conn)
			})
			.await??;

		let reservation =
			Reservation::get_by_id(reservation.id, includes, conn).await?;

		info!("created reservation {reservation:?}");

		Ok(reservation)
	}
}
