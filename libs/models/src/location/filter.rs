use std::f64;

use common::{DbConn, Error};
use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;

use super::{description, excerpt};
use crate::db::{
	approver,
	authority,
	creator,
	location,
	opening_time,
	profile,
	rejecter,
	translation,
	updater,
};
use crate::{
	BoxedCondition,
	Location,
	LocationIncludes,
	PrimitiveLocation,
	QUERY_HARD_LIMIT,
	TimeBoundsFilter,
	TimeFilter,
	ToFilter,
	manual_pagination,
};

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
	reservable: Option<ReservableFilter>,
	#[serde(flatten)]
	bounds:     Option<BoundsFilter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilter {
	pub language: String,
	pub query:    String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservableFilter {
	pub is_reservable: bool,
}

#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundsFilter {
	#[serde_as(as = "DisplayFromStr")]
	pub north_east_lat: f64,
	#[serde_as(as = "DisplayFromStr")]
	pub north_east_lng: f64,
	#[serde_as(as = "DisplayFromStr")]
	pub south_west_lat: f64,
	#[serde_as(as = "DisplayFromStr")]
	pub south_west_lng: f64,
}

impl<S> ToFilter<S> for LocationFilter
where
	S: 'static,
	diesel::dsl::Nullable<location::is_visible>: SelectableExpression<S>,
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

		if let Some(resv) = self.reservable {
			filter = Box::new(filter.and(resv.to_filter()));
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

impl Location {
	/// Search through all [`Location`]s with a given [`LocationFilter`]
	#[instrument(skip(conn))]
	pub async fn search(
		loc_filter: LocationFilter,
		time_filter: TimeFilter,
		includes: LocationIncludes,
		limit: usize,
		offset: usize,
		conn: &DbConn,
	) -> Result<(usize, bool, Vec<Self>), Error> {
		let filter = loc_filter.to_filter();
		let query = Self::joined_query(includes);

		let time_filter = if let Some(open_on_day) = time_filter.open_on_day {
			let week = open_on_day.week(chrono::Weekday::Mon);
			// I don't think blokmap will still be used in 264.000 AD so unwrap
			// should be safe
			let week_start = week.checked_first_day().unwrap();
			let week_end = week.checked_last_day().unwrap();

			let bounds_filter = TimeBoundsFilter {
				start_date: Some(week_start),
				end_date:   Some(week_end),
			};

			Box::new(time_filter.to_filter().and(bounds_filter.to_filter()))
		} else {
			time_filter.to_filter()
		};

		let locations = conn
			.interact(move |conn| {
				use crate::db::location::dsl::*;

				query
					.filter(filter)
					.filter(diesel::dsl::exists(
						opening_time::table
							.filter(time_filter)
							.filter(opening_time::location_id.eq(id))
							.select(opening_time::id),
					))
					.select((
						PrimitiveLocation::as_select(),
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
						authority::all_columns.nullable(),
						approver.fields(profile::all_columns).nullable(),
						rejecter.fields(profile::all_columns).nullable(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.order(id)
					.limit(QUERY_HARD_LIMIT)
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(l, d, e, y, a, r, c, u)| {
				Self::from_joined(includes, (l, d, e, y, a, r, c, u))
			})
			.collect::<Vec<_>>();

		manual_pagination(locations, limit, offset)
	}
}
