use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::{Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::Translation;
use crate::DbConn;
use crate::error::Error;
use crate::schema::{location, translation};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bounds {
	pub north_east_lat: f64,
	pub north_east_lng: f64,
	pub south_west_lat: f64,
	pub south_west_lng: f64,
}

#[derive(
	Clone, Debug, Deserialize, Serialize, Identifiable, Queryable, Selectable,
)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = location)]
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
	pub created_by_id:  i32,
	pub approved_by_id: Option<i32>,
	pub approved_at:    Option<NaiveDateTime>,
	pub created_at:     NaiveDateTime,
	pub updated_at:     NaiveDateTime,
}

impl Location {
	/// Get a [`Location`] by its id and include its [`Translation`]s.
	pub async fn get_by_id(
		loc_id: i32,
		conn: &DbConn,
	) -> Result<(Location, Translation, Translation), Error> {
		let result = conn
			.interact(move |conn| {
				let (description, excerpt) = diesel::alias!(
					translation as description,
					translation as excerpt
				);

				location::table
					.filter(location::id.eq(loc_id))
					.inner_join(
						description.on(location::description_id
							.eq(description.field(translation::id))),
					)
					.inner_join(excerpt.on(
						location::excerpt_id.eq(excerpt.field(translation::id)),
					))
					.select((
						location::all_columns,
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
					))
					.first(conn)
			})
			.await??;

		Ok(result)
	}

	/// Get all [`Location`]s and include their [`Translation`]s.
	pub async fn get_all(
		bounds: Bounds,
		conn: &DbConn,
	) -> Result<Vec<(Location, Translation, Translation)>, Error> {
		let locations = conn
			.interact(move |conn| {
				// Alias the translation table twice to join it twice.
				let (description, excerpt) = diesel::alias!(
					translation as description,
					translation as excerpt
				);

				// Get the bounds for the locations.
				let (north_lat, north_lng) =
					(bounds.north_east_lat, bounds.north_east_lng);

				let (south_lat, south_lng) =
					(bounds.south_west_lat, bounds.south_west_lng);

				location::table
					.filter(
						location::latitude.between(south_lat, north_lat).and(
							location::longitude.between(south_lng, north_lng),
						),
					)
					.inner_join(
						description.on(location::description_id
							.eq(description.field(translation::id))),
					)
					.inner_join(excerpt.on(
						location::excerpt_id.eq(excerpt.field(translation::id)),
					))
					.select((
						location::all_columns,
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
					))
					.load(conn)
			})
			.await??;

		Ok(locations)
	}

	/// Get all the latlng positions of the locations.
	pub async fn get_latlng_positions(
		conn: &DbConn,
	) -> Result<Vec<(f64, f64)>, Error> {
		let positions = conn
			.interact(move |conn| {
				location::table
					.select((location::latitude, location::longitude))
					.load(conn)
			})
			.await??;

		Ok(positions)
	}

	/// Delete a [`Location`] by its id.
	pub async fn delete_by_id(loc_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::location::dsl::*;

			diesel::delete(location.filter(id.eq(loc_id))).execute(conn)
		})
		.await??;

		Ok(())
	}

    /// Approve a [`Location`] by its id and profile id.
    pub async fn approve_by(loc_id: i32, profile_id: i32, conn: &DbConn) -> Result<(), Error> {
        conn.interact(move |conn| {
            use self::location::dsl::*;

            diesel::update(location.filter(id.eq(loc_id)))
                .set((
                    approved_by_id.eq(profile_id),
                    approved_at.eq(Utc::now().naive_utc()),
                ))
                .execute(conn)
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
	pub created_by_id:  i32,
}

impl NewLocation {
	/// Insert this [`NewLocation`] into the database.
	pub async fn insert(self, conn: &DbConn) -> Result<Location, Error> {
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
