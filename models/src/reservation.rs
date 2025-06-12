use chrono::{NaiveDate, NaiveDateTime};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Date};
use serde::{Deserialize, Serialize};

use crate::schema::{
	confirmer,
	creator,
	location,
	opening_time,
	reservation,
	simple_profile,
};
use crate::{PrimitiveLocation, PrimitiveOpeningTime, SimpleProfile};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationFilter {
	pub date: Option<NaiveDate>,
}

type BoxedCondition<S, T> = Box<dyn BoxableExpression<S, Pg, SqlType = T>>;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct ReservationIncludes {
	#[serde(default)]
	pub profile:      bool,
	#[serde(default)]
	pub confirmed_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(reservation))]
pub struct Reservation {
	pub reservation:  PrimitiveReservation,
	pub profile:      Option<SimpleProfile>,
	pub confirmed_by: Option<Option<SimpleProfile>>,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveReservation {
	pub id:               i32,
	pub profile_id:       i32,
	pub opening_time_id:  i32,
	pub base_block_index: i32,
	pub block_count:      i32,
	pub created_at:       NaiveDateTime,
	pub updated_at:       NaiveDateTime,
	pub confirmed_at:     Option<NaiveDateTime>,
}

impl PrimitiveReservation {
	/// Get a [`PrimitiveReservation`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(r_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let reservation = conn
			.interact(move |conn| {
				use crate::schema::reservation::dsl::*;

				reservation
					.find(r_id)
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		Ok(reservation)
	}
}

impl Reservation {
	/// Get a [`Reservation`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		r_id: i32,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let reservation: (
			PrimitiveReservation,
			Option<SimpleProfile>,
			Option<SimpleProfile>,
		) = conn
			.interact(move |conn| {
				use crate::schema::reservation::dsl::*;

				reservation
					.left_outer_join(creator.on(
						includes.profile.into_sql::<Bool>().and(
							profile_id.eq(creator.field(simple_profile::id)),
						),
					))
					.left_outer_join(confirmer.on(
						includes.confirmed_by.into_sql::<Bool>().and(
							confirmed_by.eq(
								confirmer.field(simple_profile::id).nullable(),
							),
						),
					))
					.filter(id.eq(r_id))
					.select((
						PrimitiveReservation::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						confirmer
							.fields(simple_profile::all_columns)
							.nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let reservation = Self {
			reservation:  reservation.0,
			profile:      reservation.1,
			confirmed_by: if includes.confirmed_by {
				Some(reservation.2)
			} else {
				None
			},
		};

		Ok(reservation)
	}

	/// Get all the reservations for a specific [`Location`](crate::Location)
	#[instrument(skip(conn))]
	pub async fn for_location(
		loc_id: i32,
		filter: ReservationFilter,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Vec<(PrimitiveLocation, PrimitiveOpeningTime, Self)>, Error> {
		let date_filter: BoxedCondition<_, Bool> =
			if let Some(date) = filter.date {
				Box::new(date.into_sql::<Date>().eq(opening_time::day))
			} else {
				Box::new(true.into_sql::<Bool>().eq(true))
			};

		let reservations: Vec<(PrimitiveLocation, PrimitiveOpeningTime, Self)> =
			conn.interact(move |conn| {
				location::table
					.inner_join(
						opening_time::table
							.on(opening_time::location_id.eq(location::id)),
					)
					.inner_join(
						reservation::table
							.on(reservation::opening_time_id
								.eq(opening_time::id)),
					)
					.left_outer_join(
						creator.on(includes.profile.into_sql::<Bool>().and(
							reservation::profile_id
								.eq(creator.field(simple_profile::id)),
						)),
					)
					.left_outer_join(confirmer.on(
						includes.confirmed_by.into_sql::<Bool>().and(
							reservation::confirmed_by.eq(
								confirmer.field(simple_profile::id).nullable(),
							),
						),
					))
					.filter(location::id.eq(loc_id))
					.filter(date_filter)
					.select((
						PrimitiveLocation::as_select(),
						PrimitiveOpeningTime::as_select(),
						PrimitiveReservation::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						confirmer
							.fields(simple_profile::all_columns)
							.nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(loc, time, r, cr, conf)| {
				let res = Self {
					reservation:  r,
					profile:      cr,
					confirmed_by: if includes.confirmed_by {
						Some(conf)
					} else {
						None
					},
				};

				(loc, time, res)
			})
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
	) -> Result<Vec<(PrimitiveOpeningTime, Self)>, Error> {
		let reservations: Vec<(PrimitiveOpeningTime, Self)> = conn
			.interact(move |conn| {
				opening_time::table
					.inner_join(
						reservation::table
							.on(reservation::opening_time_id
								.eq(opening_time::id)),
					)
					.left_outer_join(
						creator.on(includes.profile.into_sql::<Bool>().and(
							reservation::profile_id
								.eq(creator.field(simple_profile::id)),
						)),
					)
					.left_outer_join(confirmer.on(
						includes.confirmed_by.into_sql::<Bool>().and(
							reservation::confirmed_by.eq(
								confirmer.field(simple_profile::id).nullable(),
							),
						),
					))
					.filter(opening_time::id.eq(t_id))
					.select((
						PrimitiveOpeningTime::as_select(),
						PrimitiveReservation::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						confirmer
							.fields(simple_profile::all_columns)
							.nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(time, r, cr, conf)| {
				let res = Self {
					reservation:  r,
					profile:      cr,
					confirmed_by: if includes.confirmed_by {
						Some(conf)
					} else {
						None
					},
				};

				(time, res)
			})
			.collect();

		Ok(reservations)
	}

	/// Get all the reservations for a specific [`Profile`](crate::Profile)
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Vec<(PrimitiveLocation, PrimitiveOpeningTime, Self)>, Error> {
		let reservations: Vec<(PrimitiveLocation, PrimitiveOpeningTime, Self)> =
			conn.interact(move |conn| {
				location::table
					.inner_join(
						opening_time::table
							.on(opening_time::location_id.eq(location::id)),
					)
					.inner_join(
						reservation::table
							.on(reservation::opening_time_id
								.eq(opening_time::id)),
					)
					.inner_join(
						creator.on(reservation::profile_id
							.eq(creator.field(simple_profile::id))),
					)
					.left_outer_join(confirmer.on(
						includes.confirmed_by.into_sql::<Bool>().and(
							reservation::confirmed_by.eq(
								confirmer.field(simple_profile::id).nullable(),
							),
						),
					))
					.filter(creator.field(simple_profile::id).eq(p_id))
					.select((
						PrimitiveLocation::as_select(),
						PrimitiveOpeningTime::as_select(),
						PrimitiveReservation::as_select(),
						creator.fields(simple_profile::all_columns),
						confirmer
							.fields(simple_profile::all_columns)
							.nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(loc, time, r, cr, conf)| {
				let res = Self {
					reservation:  r,
					profile:      if includes.profile {
						Some(cr)
					} else {
						None
					},
					confirmed_by: if includes.confirmed_by {
						Some(conf)
					} else {
						None
					},
				};

				(loc, time, res)
			})
			.collect();

		Ok(reservations)
	}

	/// Get all the block (base, count) pairs a given opening time
	#[instrument(skip(conn))]
	pub async fn get_spans_for_opening_time(
		t_id: i32,
		conn: &DbConn,
	) -> Result<Vec<(i32, i32)>, Error> {
		let pairs = conn
			.interact(move |conn| {
				use crate::schema::reservation::dsl::*;

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
			use crate::schema::reservation::dsl::*;

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
