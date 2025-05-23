use std::f64;

use chrono::NaiveDateTime;
use diesel::dsl::{InnerJoinQuerySource, LeftJoinQuerySource, sql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_source::{Alias, AliasedField};
use diesel::sql_types::{Bool, Double, Nullable, Timestamp};
use serde::{Deserialize, Serialize};

use super::{
	DescriptionAlias,
	ExcerptAlias,
	FullLocationData,
	description,
	excerpt,
};
use crate::models::Location;
use crate::schema::{location, opening_time, translation};
use crate::{DbConn, Error};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationFilter {
	pub distance:   Option<f64>,
	pub center_lat: Option<f64>,
	pub center_lng: Option<f64>,

	pub name:             Option<String>,
	pub has_reservations: Option<bool>,
	pub open_on:          Option<NaiveDateTime>,

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

		if let Some(name) = self.name {
			conditions.push(Box::new(location::name.eq(name).nullable()));
		}

		if let Some(has_reservations) = self.has_reservations {
			conditions.push(Box::new(
				location::is_reservable.eq(has_reservations).nullable(),
			));
		}

		if let Some(open_on) = self.open_on {
			conditions.push(Box::new(
				open_on
					.into_sql::<Timestamp>()
					.between(opening_time::start_time, opening_time::end_time)
					.nullable()
					.and(opening_time::id.is_not_null())
					.nullable(),
			));
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
		location_filter: LocationFilter,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let mut filter: BoxedCondition =
			Box::new(true.as_sql::<Bool>().eq(true).nullable());

		if let Some(f) = location_filter.into_boxed_condition() {
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
					.select((
						location::all_columns,
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
						opening_time::all_columns.nullable(),
					))
					.get_results(conn)
			})
			.await??;

		Ok(Self::group_by_id(result))
	}
}
