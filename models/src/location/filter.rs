use std::f64;

use common::{DbConn, Error, PaginationError};
use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Double, Nullable, Text};
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;

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
	BoxedCondition,
	Location,
	LocationIncludes,
	PrimitiveLocation,
	PrimitiveTranslation,
	TimeBoundsFilter,
	TimeFilter,
	ToFilter,
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
	distance:   Option<DistanceFilter>,
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

#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistanceFilter {
	#[serde_as(as = "DisplayFromStr")]
	pub center_lat: f64,
	#[serde_as(as = "DisplayFromStr")]
	pub center_lng: f64,
	#[serde_as(as = "DisplayFromStr")]
	pub distance:   f64,
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

		if let Some(dist) = self.distance {
			filter = Box::new(filter.and(dist.to_filter()));
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
		time_filter: TimeFilter,
		includes: LocationIncludes,
		limit: usize,
		offset: usize,
		conn: &DbConn,
	) -> Result<(usize, Vec<Self>), Error> {
		let filter = loc_filter.to_filter();
		let query = Self::build_query(includes);

		let bounds_filter = if let Some(open_on_day) = time_filter.open_on_day {
			let week = open_on_day.week(chrono::Weekday::Mon);
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
			let now = chrono::Utc::now().date_naive();
			let week = now.week(chrono::Weekday::Mon);
			let week_start = week.checked_first_day().unwrap();
			let week_end = week.checked_last_day().unwrap();

			let bounds_filter = TimeBoundsFilter {
				start_date: Some(week_start),
				end_date:   Some(week_end),
			};

			bounds_filter.to_filter()
		};

		let time_filter = Box::new(time_filter.to_filter().and(bounds_filter));

		let locations = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;

				query
					.filter(filter)
					.filter(
						diesel::dsl::exists(
							opening_time::table
								.filter(time_filter)
								.filter(opening_time::location_id.eq(id))
								.select(opening_time::id)
						)
					)
					.select(
						(
							PrimitiveLocation::as_select(),
							description
								.fields(<PrimitiveTranslation as Selectable<
								Pg,
							>>::construct_selection()),
							excerpt
								.fields(<PrimitiveTranslation as Selectable<
								Pg,
							>>::construct_selection()),
							approver
								.fields(simple_profile::all_columns)
								.nullable(),
							rejecter
								.fields(simple_profile::all_columns)
								.nullable(),
							creator
								.fields(simple_profile::all_columns)
								.nullable(),
							updater
								.fields(simple_profile::all_columns)
								.nullable(),
						),
					)
					.order(id)
					.limit(1000)
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(loc, desc, exc, a, r, c, u)| {
				Location {
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
				}
			})
			.collect::<Vec<_>>();

		let total = locations.len();

		if offset >= total {
			return Err(PaginationError::OffsetTooLarge.into());
		}

		let limit = if limit > locations[offset..].len() {
			locations[offset..].len() - offset
		} else {
			limit
		};

		let locations = locations[offset..offset + limit].to_vec();

		Ok((total, locations))
	}
}
