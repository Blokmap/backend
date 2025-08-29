#[macro_use]
extern crate tracing;

use base::{BoxedCondition, ToFilter};
use chrono::NaiveDate;
use common::{DbConn, Error};
use db::{
	ConfirmerAlias,
	CreatorAlias,
	confirmer,
	creator,
	location,
	opening_time,
	profile,
	reservation,
};
use diesel::dsl::{AliasedFields, Nullable};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Date};
use primitives::{
	PrimitiveLocation,
	PrimitiveOpeningTime,
	PrimitiveProfile,
	PrimitiveReservation,
};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct Reservation {
	#[diesel(embed)]
	pub primitive:    PrimitiveReservation,
	#[diesel(embed)]
	pub opening_time: PrimitiveOpeningTime,
	#[diesel(embed)]
	pub location:     PrimitiveLocation,
	#[diesel(select_expression = profile_fragment())]
	pub profile:      Option<PrimitiveProfile>,
	#[diesel(select_expression = confirmed_by_fragment())]
	pub confirmed_by: Option<PrimitiveProfile>,
}

#[allow(non_camel_case_types)]
type profile_fragment = Nullable<
	AliasedFields<CreatorAlias, <profile::table as Table>::AllColumns>,
>;
fn profile_fragment() -> profile_fragment {
	creator.fields(profile::all_columns).nullable()
}

#[allow(non_camel_case_types)]
type confirmed_by_fragment = Nullable<
	AliasedFields<ConfirmerAlias, <profile::table as Table>::AllColumns>,
>;
fn confirmed_by_fragment() -> confirmed_by_fragment {
	confirmer.fields(profile::all_columns).nullable()
}

impl Reservation {
	/// Build a query with all required (dynamic) joins to select a full
	/// reservation data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: ReservationIncludes) -> _ {
		let inc_profile: bool = includes.profile;
		let inc_confirmed: bool = includes.confirmed_by;

		reservation::table
			.inner_join(
				opening_time::table
					.on(reservation::opening_time_id.eq(opening_time::id)),
			)
			.inner_join(
				location::table.on(opening_time::location_id.eq(location::id)),
			)
			.left_join(creator.on(
				inc_profile.into_sql::<Bool>().and(
					reservation::profile_id.eq(creator.field(profile::id)),
				),
			))
			.left_join(
				confirmer.on(inc_confirmed.into_sql::<Bool>().and(
					reservation::confirmed_by
						.eq(confirmer.field(profile::id).nullable()),
				)),
			)
	}

	/// Get a [`Reservation`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		r_id: i32,
		includes: ReservationIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::query(includes);

		let reservation = conn
			.interact(move |conn| {
				use self::reservation::dsl::*;

				query
					.filter(id.eq(r_id))
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

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
		let query = Self::query(includes);

		let reservations = conn
			.interact(move |conn| {
				query
					.filter(location::id.eq(loc_id))
					.filter(filter)
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

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
		let query = Self::query(includes);

		let reservations = conn
			.interact(move |conn| {
				query
					.filter(opening_time::id.eq(t_id))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

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
		let query = Self::query(includes);

		let reservations = conn
			.interact(move |conn| {
				query
					.filter(reservation::profile_id.eq(p_id))
					.filter(filter)
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

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
