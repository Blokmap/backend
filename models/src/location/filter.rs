use std::f64;

use chrono::{NaiveDate, NaiveTime};
use common::{DbConn, Error};
use diesel::dsl::{InnerJoinQuerySource, LeftJoinQuerySource, sql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_source::{Alias, AliasedField};
use diesel::sql_types::{Bool, Date, Double, Nullable, Text, Time};
use serde::{Deserialize, Serialize};

use super::{DescriptionAlias, ExcerptAlias, description, excerpt};
use crate::schema::{location, opening_time, translation};
use crate::{Location, PaginationOptions};

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

/// Abandon all hope, ye who enter here
type Source = LeftJoinQuerySource<
	InnerJoinQuerySource<
		InnerJoinQuerySource<
			location::table,
			Alias<DescriptionAlias>,
			diesel::dsl::Eq<
				location::description_id,
				AliasedField<DescriptionAlias, translation::id>,
			>,
		>,
		Alias<ExcerptAlias>,
		diesel::dsl::Eq<
			location::excerpt_id,
			AliasedField<ExcerptAlias, translation::id>,
		>,
	>,
	opening_time::table,
	diesel::dsl::Eq<
		diesel::dsl::Nullable<opening_time::location_id>,
		diesel::dsl::Nullable<location::id>,
	>,
>;

type BoxedCondition =
	Box<dyn BoxableExpression<Source, Pg, SqlType = Nullable<Bool>>>;

impl LocationFilter {
	fn into_boxed_condition(self) -> Option<BoxedCondition> {
		let mut conditions: Vec<BoxedCondition> = vec![];

		if let Some(query) = self.query {
			let language = self
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

			conditions.push(Box::new(
				name_filter.or(desc_filter).or(exc_filter).nullable(),
			));
		}

		if let Some(dist) = self.distance {
			// The route controller guarantees that if one param is present,
			// all of them are
			let lat = self.center_lat.unwrap();
			let lng = self.center_lng.unwrap();

			conditions.push(Box::new(
				sql::<Double>("2 * 6371 * asin(sqrt(1 - cos(radians( ")
					.bind::<Double, _>(lat)
					.sql(
						" ) - radians( latitude )) + cos(radians( latitude )) \
						 * cos(radians( ",
					)
					.bind::<Double, _>(lat)
					.sql(" )) * (1 - cos(radians( ")
					.bind::<Double, _>(lng)
					.sql(" ) - radians( longitude ))) / 2))")
					.le(dist)
					.nullable(),
			));
		}

		if let Some(is_reservable) = self.is_reservable {
			conditions.push(Box::new(
				location::is_reservable.eq(is_reservable).nullable(),
			));
		}

		if let Some(open_on_day) = self.open_on_day {
			conditions.push(Box::new(
				open_on_day
					.into_sql::<Date>()
					.eq(opening_time::day)
					.and(opening_time::id.is_not_null())
					.nullable(),
			));

			if let Some(open_on_time) = self.open_on_time {
				conditions.push(Box::new(
					open_on_time
						.into_sql::<Time>()
						.between(
							opening_time::start_time,
							opening_time::end_time,
						)
						.and(opening_time::id.is_not_null())
						.nullable(),
				));
			}
		}

		if let Some(north_lat) = self.north_east_lat {
			// The route controller guarantees that if one bound is present,
			// all of them are
			let north_lng = self.north_east_lng.unwrap();
			let south_lat = self.south_west_lat.unwrap();
			let south_lng = self.south_west_lng.unwrap();

			conditions.push(Box::new(
				location::latitude.between(south_lat, north_lat).and(
					location::longitude
						.between(south_lng, north_lng)
						.nullable(),
				),
			));
		}

		conditions.into_iter().fold(
			None,
			|conditions: Option<BoxedCondition>, condition| {
				Some(match conditions {
					Some(cs) => Box::new(cs.and(condition)),
					None => condition,
				})
			},
		)
	}
}

impl Location {
	/// Search through all [`Location`]s with a given [`LocationFilter`]
	///
	/// # Errors
	#[instrument(skip(conn))]
	pub async fn search(
		loc_filter: LocationFilter,
		pagination: PaginationOptions,
		conn: &DbConn,
	) -> Result<Vec<PartialLocation>, Error> {
		let mut filter: BoxedCondition =
			Box::new(location::is_visible.eq(true).nullable());

		if let Some(f) = loc_filter.into_boxed_condition() {
			filter = Box::new(filter.and(f));
		}

		let result = conn
			.interact(move |conn| {
				location::table
					.inner_join(
						description.on(location::description_id
							.eq(description.field(translation::id))),
					)
					.inner_join(excerpt.on(
						location::excerpt_id.eq(excerpt.field(translation::id)),
					))
					.left_outer_join(opening_time::table)
					.filter(filter)
					.limit(pagination.per_page.into())
					.select(PartialLocation::as_select())
					.distinct()
					.get_results(conn)
			})
			.await??;

		Ok(result)
	}
}
