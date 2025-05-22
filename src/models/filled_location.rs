use std::f64;

use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::dsl::sql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Double, Jsonb, Nullable, Timestamp};
use serde::{Deserialize, Serialize};

use self::schema::filled_location;
use crate::{DbConn, Error};

pub(crate) mod schema {
	use diesel::table;

	table! {
		filled_location (id) {
			id -> Int4,
			name -> Text,
			seat_count -> Int4,
			is_reservable -> Bool,
			is_visible -> Bool,
			street -> Text,
			number -> Text,
			zip -> Text,
			city -> Text,
			province -> Text,
			latitude -> Float8,
			longitude -> Float8,
			created_by_id -> Int4,
			approved_by_id -> Nullable<Int4>,
			approved_at -> Nullable<Timestamp>,
			created_at -> Timestamp,
			updated_at -> Timestamp,
			description -> Jsonb,
			excerpt -> Jsonb,
			opening_times -> Jsonb,
		}
	}
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = filled_location)]
#[diesel(check_for_backend(Pg))]
pub struct FilledLocation {
	pub id:             i32,
	pub name:           String,
	pub seat_count:     i32,
	pub is_reservable:  bool,
	pub is_visible:     bool,
	pub street:         String,
	pub number:         String,
	pub zip:            String,
	pub city:           String,
	pub province:       String,
	pub latitude:       f64,
	pub longitude:      f64,
	pub created_by_id:  i32,
	pub approved_by_id: Option<i32>,
	pub approved_at:    Option<NaiveDateTime>,
	pub created_at:     NaiveDateTime,
	pub updated_at:     NaiveDateTime,
	pub description:    FilledTranslation,
	pub excerpt:        FilledTranslation,
	pub opening_times:  FilledOpeningTimes,
}

#[derive(Clone, Debug, Deserialize, FromSqlRow, Serialize)]
pub struct FilledTranslation {
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

impl<DB> FromSql<Jsonb, DB> for FilledTranslation
where
	DB: Backend,
	serde_json::Value: FromSql<Jsonb, DB>,
{
	fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
		let value = <serde_json::Value as FromSql<Jsonb, DB>>::from_sql(bytes)?;
		Ok(serde_json::from_value(value)?)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilledOpeningTime {
	pub start_time:    NaiveDateTime,
	pub end_time:      NaiveDateTime,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
	pub created_at:    NaiveDateTime,
	pub updated_at:    NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, FromSqlRow, Serialize)]
pub struct FilledOpeningTimes(Vec<FilledOpeningTime>);

impl<DB> FromSql<Jsonb, DB> for FilledOpeningTimes
where
	DB: Backend,
	serde_json::Value: FromSql<Jsonb, DB>,
{
	fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
		let value = <serde_json::Value as FromSql<Jsonb, DB>>::from_sql(bytes)?;
		let inner: Vec<FilledOpeningTime> = serde_json::from_value(value)?;

		Ok(Self(inner))
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationFilter {
	pub distance:         Option<String>,
	pub name:             Option<String>,
	pub has_reservations: Option<bool>,
	pub open_on:          Option<NaiveDateTime>,

	pub north_east_lat: Option<f64>,
	pub north_east_lng: Option<f64>,
	pub south_west_lat: Option<f64>,
	pub south_west_lng: Option<f64>,
}

type BoxedCondition = Box<
	dyn BoxableExpression<filled_location::table, Pg, SqlType = Nullable<Bool>>,
>;

impl LocationFilter {
	fn into_boxed_condition(self) -> Result<Option<BoxedCondition>, Error> {
		let mut conditions: Vec<BoxedCondition> = vec![];

		if let Some(distance) = self.distance {
			let parts: Vec<&str> = distance.split('_').collect();
			if parts.len() != 3 {
				return Err(Error::ValidationError(
					"expected parameter to be <lat>-<lng>-<dist>".into(),
				));
			}

			let lat: f64 = parts[0].parse().map_err(|_| {
				Error::ValidationError("expected lat to be f64".into())
			})?;
			let lng: f64 = parts[1].parse().map_err(|_| {
				Error::ValidationError("expected lng to be f64".into())
			})?;
			let dist: f64 = parts[2].parse().map_err(|_| {
				Error::ValidationError("expected dist to be f64".into())
			})?;

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
			conditions
				.push(Box::new(filled_location::name.eq(name).nullable()));
		}

		if let Some(has_reservations) = self.has_reservations {
			conditions.push(Box::new(
				filled_location::is_reservable.eq(has_reservations).nullable(),
			));
		}

		if let Some(open_on) = self.open_on {
			conditions.push(Box::new(
				sql::<Bool>(
					"EXISTS(SELECT 1 FROM \
					 jsonb_array_elements(filled_location.opening_times) AS t \
					 WHERE ",
				)
				.bind::<Timestamp, _>(open_on)
				.sql(
					" BETWEEN CAST(t->>'start_time' AS TIMESTAMP) AND \
					 CAST(t->>'end_time' AS TIMESTAMP))",
				)
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
				filled_location::latitude.between(south_lat, north_lat).and(
					filled_location::longitude
						.between(south_lng, north_lng)
						.nullable(),
				),
			));
		}

		Ok(conditions.into_iter().fold(
			None,
			|conditions: Option<BoxedCondition>, condition| {
				Some(match conditions {
					Some(cs) => Box::new(cs.and(condition)),
					None => condition,
				})
			},
		))
	}
}

impl FilledLocation {
	/// Search through all [`FilledLocation`] with a given [`LocationFilter`]
	///
	/// # Errors
	#[instrument(skip(conn))]
	pub async fn search(
		location_filter: LocationFilter,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let mut filter: BoxedCondition =
			Box::new(true.as_sql::<Bool>().eq(true).nullable());

		if let Some(f) = location_filter.into_boxed_condition()? {
			filter = Box::new(filter.and(f));
		}

		let result = conn
			.interact(move |conn| {
				filled_location::table
					.filter(filter)
					.select(FilledLocation::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(result)
	}
}
