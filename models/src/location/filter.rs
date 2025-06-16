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
#[serde(rename_all = "camelCase")]
pub struct LocationFilter {
	pub language: Option<String>,
	pub query:    Option<String>,

	pub open_on_day:  Option<NaiveDate>,
	pub open_on_time: Option<NaiveTime>,

	pub center_lat: Option<f64>,
	pub center_lng: Option<f64>,
	pub distance:   Option<f64>,

	pub is_reservable: Option<bool>,

	pub north_east_lat: Option<f64>,
	pub north_east_lng: Option<f64>,
	pub south_west_lat: Option<f64>,
	pub south_west_lng: Option<f64>,
}

type BoxedCondition<S> =
	Box<dyn BoxableExpression<S, Pg, SqlType = Nullable<Bool>>>;

impl Location {
	/// Search through all [`Location`]s with a given [`LocationFilter`]
	#[instrument(skip(conn))]
	#[allow(clippy::too_many_lines)]
	pub async fn search(
		loc_filter: LocationFilter,
		includes: LocationIncludes,
		per_page: i64,
		offset: i64,
		conn: &DbConn,
	) -> Result<(i64, Vec<FullLocationData>), Error> {
		let mut filter: BoxedCondition<_> =
			Box::new(location::is_visible.eq(true).nullable());

		if let Some(query) = loc_filter.query.clone() {
			let language = loc_filter
				.language
				.clone()
				.unwrap_or_else(|| String::from("en"))
				.to_ascii_lowercase();

			let dyn_description = diesel_dynamic_schema::table("description");
			let dyn_excerpt = diesel_dynamic_schema::table("excerpt");

			let name_filter = sql::<Bool>("")
				.bind::<Text, _>(location::name)
				.sql(" % ")
				.bind::<Text, _>(query.clone());

			let desc_filter = sql::<Bool>("")
				.bind::<Text, _>(dyn_description.column(language.clone()))
				.sql(" % ")
				.bind::<Text, _>(query.clone())
				.nullable();

			let exc_filter = sql::<Bool>("")
				.bind::<Text, _>(dyn_excerpt.column(language))
				.sql(" % ")
				.bind::<Text, _>(query)
				.nullable();

			filter = Box::new(
				filter
					.and(name_filter.or(desc_filter).or(exc_filter).nullable()),
			);
		}

		if let Some(dist) = loc_filter.distance {
			// The route controller guarantees that if one param is present,
			// all of them are
			let lat = loc_filter.center_lat.unwrap();
			let lng = loc_filter.center_lng.unwrap();

			filter = Box::new(
				filter.and(
					sql::<Double>("2 * 6371 * asin(sqrt(1 - cos(radians( ")
						.bind::<Double, _>(lat)
						.sql(
							" ) - radians( latitude )) + cos(radians( \
							 latitude )) * cos(radians( ",
						)
						.bind::<Double, _>(lat)
						.sql(" )) * (1 - cos(radians( ")
						.bind::<Double, _>(lng)
						.sql(" ) - radians( longitude ))) / 2))")
						.le(dist)
						.nullable(),
				),
			);
		}

		if let Some(is_reservable) = loc_filter.is_reservable {
			filter = Box::new(
				filter
					.and(location::is_reservable.eq(is_reservable).nullable()),
			);
		}

		if let Some(open_on_day) = loc_filter.open_on_day {
			filter = Box::new(
				filter.and(
					open_on_day
						.into_sql::<Date>()
						.eq(opening_time::day)
						.and(opening_time::id.is_not_null())
						.nullable(),
				),
			);

			if let Some(open_on_time) = loc_filter.open_on_time {
				filter = Box::new(
					filter.and(
						open_on_time
							.into_sql::<Time>()
							.between(
								opening_time::start_time,
								opening_time::end_time,
							)
							.and(opening_time::id.is_not_null())
							.nullable(),
					),
				);
			}
		}

		if let Some(north_lat) = loc_filter.north_east_lat {
			// The route controller guarantees that if one bound is present,
			// all of them are
			let north_lng = loc_filter.north_east_lng.unwrap();
			let south_lat = loc_filter.south_west_lat.unwrap();
			let south_lng = loc_filter.south_west_lng.unwrap();

			filter = Box::new(
				filter.and(
					location::latitude.between(south_lat, north_lat).and(
						location::longitude
							.between(south_lng, north_lng)
							.nullable(),
					),
				),
			);
		}

		let query = {
			use crate::schema::location::dsl::*;

			location
				.inner_join(
					description
						.on(description_id
							.eq(description.field(translation::id))),
				)
				.inner_join(
					excerpt.on(excerpt_id.eq(excerpt.field(translation::id))),
				)
				.left_outer_join(opening_time::table)
				.left_outer_join(
					approver.on(includes
						.approved_by
						.into_sql::<Bool>()
						.and(approved_by.eq(
							approver.field(simple_profile::id).nullable(),
						))),
				)
				.left_outer_join(
					rejecter.on(includes
						.rejected_by
						.into_sql::<Bool>()
						.and(rejected_by.eq(
							rejecter.field(simple_profile::id).nullable(),
						))),
				)
				.left_outer_join(
					creator.on(includes.created_by.into_sql::<Bool>().and(
						created_by
							.eq(creator.field(simple_profile::id).nullable()),
					)),
				)
				.left_outer_join(
					updater.on(includes.updated_by.into_sql::<Bool>().and(
						updated_by
							.eq(updater.field(simple_profile::id).nullable()),
					)),
				)
				.filter(filter)
		};

		let total: i64 = conn
			.interact(move |conn| {
				use diesel::dsl::count;

				use crate::schema::location::dsl::*;

				query.select(count(id)).first(conn)
			})
			.await??;

		let mut filter: BoxedCondition<_> =
			Box::new(location::is_visible.eq(true).nullable());

		if let Some(query) = loc_filter.query {
			let language = loc_filter
				.language
				.unwrap_or_else(|| String::from("en"))
				.to_ascii_lowercase();

			let dyn_description = diesel_dynamic_schema::table("description");
			let dyn_excerpt = diesel_dynamic_schema::table("excerpt");

			let name_filter = sql::<Bool>("")
				.bind::<Text, _>(location::name)
				.sql(" % ")
				.bind::<Text, _>(query.clone());

			let desc_filter = sql::<Bool>("")
				.bind::<Text, _>(dyn_description.column(language.clone()))
				.sql(" % ")
				.bind::<Text, _>(query.clone())
				.nullable();

			let exc_filter = sql::<Bool>("")
				.bind::<Text, _>(dyn_excerpt.column(language))
				.sql(" % ")
				.bind::<Text, _>(query)
				.nullable();

			filter = Box::new(
				filter
					.and(name_filter.or(desc_filter).or(exc_filter).nullable()),
			);
		}

		if let Some(dist) = loc_filter.distance {
			// The route controller guarantees that if one param is present,
			// all of them are
			let lat = loc_filter.center_lat.unwrap();
			let lng = loc_filter.center_lng.unwrap();

			filter = Box::new(
				filter.and(
					sql::<Double>("2 * 6371 * asin(sqrt(1 - cos(radians( ")
						.bind::<Double, _>(lat)
						.sql(
							" ) - radians( latitude )) + cos(radians( \
							 latitude )) * cos(radians( ",
						)
						.bind::<Double, _>(lat)
						.sql(" )) * (1 - cos(radians( ")
						.bind::<Double, _>(lng)
						.sql(" ) - radians( longitude ))) / 2))")
						.le(dist)
						.nullable(),
				),
			);
		}

		if let Some(is_reservable) = loc_filter.is_reservable {
			filter = Box::new(
				filter
					.and(location::is_reservable.eq(is_reservable).nullable()),
			);
		}

		if let Some(open_on_day) = loc_filter.open_on_day {
			filter = Box::new(
				filter.and(
					open_on_day
						.into_sql::<Date>()
						.eq(opening_time::day)
						.and(opening_time::id.is_not_null())
						.nullable(),
				),
			);

			if let Some(open_on_time) = loc_filter.open_on_time {
				filter = Box::new(
					filter.and(
						open_on_time
							.into_sql::<Time>()
							.between(
								opening_time::start_time,
								opening_time::end_time,
							)
							.and(opening_time::id.is_not_null())
							.nullable(),
					),
				);
			}
		}

		if let Some(north_lat) = loc_filter.north_east_lat {
			// The route controller guarantees that if one bound is present,
			// all of them are
			let north_lng = loc_filter.north_east_lng.unwrap();
			let south_lat = loc_filter.south_west_lat.unwrap();
			let south_lng = loc_filter.south_west_lng.unwrap();

			filter = Box::new(
				filter.and(
					location::latitude.between(south_lat, north_lat).and(
						location::longitude
							.between(south_lng, north_lng)
							.nullable(),
					),
				),
			);
		}

		let query = {
			use crate::schema::location::dsl::*;

			location
				.inner_join(
					description
						.on(description_id
							.eq(description.field(translation::id))),
				)
				.inner_join(
					excerpt.on(excerpt_id.eq(excerpt.field(translation::id))),
				)
				.left_outer_join(opening_time::table)
				.left_outer_join(
					approver.on(includes
						.approved_by
						.into_sql::<Bool>()
						.and(approved_by.eq(
							approver.field(simple_profile::id).nullable(),
						))),
				)
				.left_outer_join(
					rejecter.on(includes
						.rejected_by
						.into_sql::<Bool>()
						.and(rejected_by.eq(
							rejecter.field(simple_profile::id).nullable(),
						))),
				)
				.left_outer_join(
					creator.on(includes.created_by.into_sql::<Bool>().and(
						created_by
							.eq(creator.field(simple_profile::id).nullable()),
					)),
				)
				.left_outer_join(
					updater.on(includes.updated_by.into_sql::<Bool>().and(
						updated_by
							.eq(updater.field(simple_profile::id).nullable()),
					)),
				)
				.filter(filter)
		};

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
					.limit(per_page)
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
