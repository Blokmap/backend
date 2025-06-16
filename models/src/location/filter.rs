use std::f64;

use chrono::{NaiveDate, NaiveTime};
use common::{DbConn, Error};
use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Date, Double, Nullable, Text, Time};
use serde::{Deserialize, Serialize};

use super::{description, excerpt};
use crate::schema::{
	approver,
	creator,
	location,
	opening_time,
	rejecter,
	simple_profile,
	translation,
	updater,
};
use crate::{
	FullLocationData,
	Location,
	LocationIncludes,
	PrimitiveLocation,
	PrimitiveOpeningTime,
	PrimitiveTranslation,
};

type BoxedCondition<S, T = Nullable<Bool>> =
	Box<dyn BoxableExpression<S, Pg, SqlType = T>>;

trait ToFilter<S> {
	type SqlType;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType>;
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(table_name = location)]
#[diesel(check_for_backend(Pg))]
pub struct PartialLocation {
	pub id:        i32,
	pub name:      String,
	pub latitude:  f64,
	pub longitude: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocationFilter {
	#[serde(flatten)]
	query:      Option<QueryFilter>,
	#[serde(flatten)]
	time:       Option<TimeFilter>,
	#[serde(flatten)]
	distance:   Option<DistanceFilter>,
	#[serde(flatten)]
	reservable: Option<ReservableFilter>,
	#[serde(flatten)]
	bounds:     Option<BoundsFilter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QueryFilter {
	pub language: String,
	pub query:    String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TimeFilter {
	pub open_on_day:  NaiveDate,
	pub open_on_time: Option<NaiveTime>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct DistanceFilter {
	pub center_lat: f64,
	pub center_lng: f64,
	pub distance:   f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ReservableFilter {
	pub is_reservable: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BoundsFilter {
	pub north_east_lat: f64,
	pub north_east_lng: f64,
	pub south_west_lat: f64,
	pub south_west_lng: f64,
}

impl<S> ToFilter<S> for LocationFilter
where
	S: 'static,
	diesel::dsl::Nullable<location::is_visible>: SelectableExpression<S>,
	diesel::dsl::Nullable<opening_time::id>: SelectableExpression<S>,
	diesel::dsl::Nullable<opening_time::day>: SelectableExpression<S>,
	diesel::dsl::Nullable<opening_time::start_time>: SelectableExpression<S>,
	diesel::dsl::Nullable<opening_time::end_time>: SelectableExpression<S>,
	location::latitude: SelectableExpression<S>,
	location::longitude: SelectableExpression<S>,
	location::is_reservable: SelectableExpression<S>,
{
	type SqlType = Nullable<Bool>;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		let mut filter: BoxedCondition<S, Self::SqlType> =
			Box::new(location::is_visible.nullable().eq(true));

		if let Some(query) = self.query.clone() {
			filter = Box::new(filter.and(query.to_filter()));
		}

		if let Some(dist) = self.distance {
			filter = Box::new(filter.and(dist.to_filter()));
		}

		if let Some(resv) = self.reservable {
			filter = Box::new(filter.and(resv.to_filter()));
		}

		if let Some(time) = self.time {
			filter = Box::new(filter.and(time.to_filter()));
		}

		if let Some(bounds) = self.bounds {
			filter = Box::new(filter.and(bounds.to_filter()));
		}

		filter
	}
}

impl<S> ToFilter<S> for QueryFilter {
	type SqlType = Bool;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		let language = self.language.clone().to_ascii_lowercase();

		let dyn_description = diesel_dynamic_schema::table("description");
		let dyn_excerpt = diesel_dynamic_schema::table("excerpt");

		let name_filter = sql::<Bool>("")
			.bind::<Text, _>(location::name)
			.sql(" % ")
			.bind::<Text, _>(self.query.clone());

		let desc_filter = sql::<Bool>("")
			.bind::<Text, _>(dyn_description.column(language.clone()))
			.sql(" % ")
			.bind::<Text, _>(self.query.clone());

		let exc_filter = sql::<Bool>("")
			.bind::<Text, _>(dyn_excerpt.column(language))
			.sql(" % ")
			.bind::<Text, _>(self.query.clone());

		Box::new(name_filter.or(desc_filter).or(exc_filter))
	}
}

impl<S> ToFilter<S> for TimeFilter
where
	S: 'static,
	diesel::dsl::Nullable<opening_time::id>: SelectableExpression<S>,
	diesel::dsl::Nullable<opening_time::day>: SelectableExpression<S>,
	diesel::dsl::Nullable<opening_time::start_time>: SelectableExpression<S>,
	diesel::dsl::Nullable<opening_time::end_time>: SelectableExpression<S>,
{
	type SqlType = Nullable<Bool>;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		let mut filter: BoxedCondition<S, Self::SqlType> = Box::new(
			self.open_on_day
				.into_sql::<Nullable<Date>>()
				.eq(opening_time::day.nullable()),
		);

		if let Some(open_on_time) = self.open_on_time {
			filter = Box::new(filter.and(
				open_on_time.into_sql::<Nullable<Time>>().between(
					opening_time::start_time.nullable(),
					opening_time::end_time.nullable(),
				),
			));
		}

		filter
	}
}

impl<S> ToFilter<S> for DistanceFilter
where
	location::latitude: SelectableExpression<S>,
	location::longitude: SelectableExpression<S>,
{
	type SqlType = Bool;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		Box::new(
			sql::<Double>("2 * 6371 * asin(sqrt(1 - cos(radians( ")
				.bind::<Double, _>(self.center_lat)
				.sql(
					" ) - radians( latitude )) + cos(radians( latitude )) * \
					 cos(radians( ",
				)
				.bind::<Double, _>(self.center_lat)
				.sql(" )) * (1 - cos(radians( ")
				.bind::<Double, _>(self.center_lng)
				.sql(" ) - radians( longitude ))) / 2))")
				.le(self.distance),
		)
	}
}

impl<S> ToFilter<S> for ReservableFilter
where
	location::is_reservable: SelectableExpression<S>,
{
	type SqlType = Bool;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		Box::new(location::is_reservable.eq(self.is_reservable))
	}
}

impl<S> ToFilter<S> for BoundsFilter
where
	location::latitude: SelectableExpression<S>,
	location::longitude: SelectableExpression<S>,
{
	type SqlType = Bool;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType> {
		Box::new(
			location::latitude
				.between(self.south_west_lat, self.north_east_lat)
				.and(
					location::longitude
						.between(self.south_west_lng, self.north_east_lng),
				),
		)
	}
}

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl Location {
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn build_query(includes: LocationIncludes) -> _ {
		let inc_approved_by: bool = includes.approved_by;
		let inc_rejected_by: bool = includes.rejected_by;
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		crate::schema::location::dsl::location
			.inner_join(
				description.on(crate::schema::location::dsl::description_id
					.eq(description.field(translation::id))),
			)
			.inner_join(
				excerpt.on(crate::schema::location::dsl::excerpt_id
					.eq(excerpt.field(translation::id))),
			)
			.left_outer_join(crate::schema::opening_time::table)
			.left_outer_join(
				approver.on(inc_approved_by.into_sql::<Bool>().and(
					crate::schema::location::dsl::approved_by
						.eq(approver.field(simple_profile::id).nullable()),
				)),
			)
			.left_outer_join(
				rejecter.on(inc_rejected_by.into_sql::<Bool>().and(
					crate::schema::location::dsl::rejected_by
						.eq(rejecter.field(simple_profile::id).nullable()),
				)),
			)
			.left_outer_join(
				creator.on(inc_created_by.into_sql::<Bool>().and(
					crate::schema::location::dsl::created_by
						.eq(creator.field(simple_profile::id).nullable()),
				)),
			)
			.left_outer_join(
				updater.on(inc_updated_by.into_sql::<Bool>().and(
					crate::schema::location::dsl::updated_by
						.eq(updater.field(simple_profile::id).nullable()),
				)),
			)
	}

	/// Search through all [`Location`]s with a given [`LocationFilter`]
	#[instrument(skip(conn))]
	pub async fn search(
		loc_filter: LocationFilter,
		includes: LocationIncludes,
		limit: i64,
		offset: i64,
		conn: &DbConn,
	) -> Result<(i64, Vec<FullLocationData>), Error> {
		let filter = loc_filter.to_filter();
		let query = Self::build_query(includes).filter(filter);

		let total: i64 = conn
			.interact(|conn| {
				use diesel::dsl::count_star;

				query.select(count_star()).first(conn)
			})
			.await??;

		let filter = loc_filter.to_filter();
		let query = Self::build_query(includes).filter(filter);

		let locations = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;

				query
					.select((
						PrimitiveLocation::as_select(),
						description.fields(
							<
								PrimitiveTranslation as Selectable<Pg>
							>::construct_selection()
						),
						excerpt.fields(
							<
								PrimitiveTranslation as Selectable<Pg>
							>::construct_selection()
						),
						approver.fields(simple_profile::all_columns).nullable(),
						rejecter.fields(simple_profile::all_columns).nullable(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
						<
							PrimitiveOpeningTime as Selectable<Pg>
						>
						::construct_selection().nullable(),
					))
					.order(id)
					.limit(limit)
					.offset(offset)
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(loc, desc, exc, a, r, c, u, t)| {
				let loc = Location {
					location:    loc,
					description: desc,
					excerpt:     exc,
					approved_by: if includes.approved_by {
						Some(a)
					} else {
						None
					},
					rejected_by: if includes.rejected_by {
						Some(r)
					} else {
						None
					},
					created_by:  if includes.created_by {
						Some(c)
					} else {
						None
					},
					updated_by:  if includes.updated_by {
						Some(u)
					} else {
						None
					},
				};

				(loc, t)
			})
			.collect();

		Ok((total, Self::group_by_id(locations)))
	}
}
