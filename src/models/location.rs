use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::Translation;
use crate::DbConn;
use crate::error::Error;
use crate::schema::{location, translation};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = crate::schema::location)]
#[serde(rename_all = "camelCase")]
pub struct Location {
	pub id:             i32,
	pub name:           String,
	pub description_id: i32,
	pub excerpt_id:     i32,
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
	pub created_at:     DateTime<Utc>,
	pub updated_at:     DateTime<Utc>,
}

impl Location {
	/// Get a [`Location`] by its id.
	pub(crate) async fn get_by_id(
		loc_id: i32,
		conn: &DbConn,
	) -> Result<(Location, Translation, Translation), Error> {
		// Create explicit aliases for translation table
		let description = diesel::alias!(translation as description);
		let excerpt = diesel::alias!(translation as excerpt);

		let result = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;

				// Base query
				let query = location.filter(id.eq(loc_id)).into_boxed();

				// First left join for description translation
				let query =
					query.left_join(description.on(
						description_id.eq(description.field(translation::id)),
					));

				// Second left join for excerpt translation
				let query = query.left_join(
					excerpt.on(excerpt_id.eq(excerpt.field(translation::id))),
				);

				// Select all three entities with nullables
				query
					.select((
						Location::as_select(),
						Translation::as_select(),
						Translation::as_select(),
					))
					.first(conn)
			})
			.await??;

		Ok(result)
	}

	/// Get all [`Location`]s.
	pub(crate) async fn get_all(conn: &DbConn) -> Result<Vec<Location>, Error> {
		let locations = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				location.select(Location::as_select()).load(conn)
			})
			.await??;

		Ok(locations)
	}

	/// Delete a [`Location`] by its id.
	pub(crate) async fn delete_by_id(
		loc_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::location::dsl::*;

			diesel::delete(location.filter(id.eq(loc_id))).execute(conn)
		})
		.await??;

		Ok(())
	}
}

#[derive(Queryable, Identifiable, Associations, Serialize, Debug)]
#[diesel(belongs_to(Location))]
#[diesel(table_name = crate::schema::opening_time)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTime {
	pub id:            i32,
	pub location_id:   i32,
	pub start_time:    DateTime<Utc>,
	pub end_time:      DateTime<Utc>,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
	pub created_at:    DateTime<Utc>,
	pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::location)]
#[serde(rename_all = "camelCase")]
pub struct NewLocation {
	pub name:           String,
	pub description_id: i32,
	pub excerpt_id:     i32,
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
}

impl NewLocation {
	/// Insert this [`NewLocation`] into the database.
	pub(crate) async fn insert(self, conn: &DbConn) -> Result<Location, Error> {
		let location = conn
			.interact(|conn| {
				use self::location::dsl::*;

				diesel::insert_into(location)
					.values(self)
					.returning(Location::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(location)
	}
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::location)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocation {
	pub name:          Option<String>,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
	pub is_visible:    Option<bool>,
	pub street:        Option<String>,
	pub number:        Option<String>,
	pub zip:           Option<String>,
	pub city:          Option<String>,
	pub province:      Option<String>,
	pub latitude:      Option<f64>,
	pub longitude:     Option<f64>,
}

impl UpdateLocation {
	/// Update this [`Location`] in the database.
	pub(crate) async fn update(
		self,
		loc_id: i32,
		conn: &DbConn,
	) -> Result<Location, Error> {
		let location = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				diesel::update(location.filter(id.eq(loc_id)))
					.set(self)
					.returning(Location::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(location)
	}
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::opening_time)]
#[serde(rename_all = "camelCase")]
pub struct NewOpeningTime {
	pub location_id:   i32,
	pub start_time:    DateTime<Utc>,
	pub end_time:      DateTime<Utc>,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
}
