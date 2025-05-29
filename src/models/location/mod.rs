use std::collections::HashMap;
use std::hash::Hash;

use chrono::{NaiveDateTime, Utc};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::{Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::{OpeningTime, Translation};
use crate::DbConn;
use crate::error::Error;
use crate::schema::{location, opening_time, location_image, translation};

mod filter;

pub use filter::*;

diesel::alias!(
	translation as description: DescriptionAlias,
	translation as excerpt: ExcerptAlias,
);

pub type FullLocationData =
	(Location, Translation, Translation, Vec<OpeningTime>);

#[derive(
	Clone, Debug, Deserialize, Serialize, Identifiable, Queryable, Selectable,
)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = location)]
#[diesel(check_for_backend(Pg))]
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

impl Hash for Location {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state) }
}

impl PartialEq for Location {
	fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for Location {}

impl Location {
	fn group_by_id(
		data: Vec<(Location, Translation, Translation, Option<OpeningTime>)>,
	) -> Vec<FullLocationData> {
		let mut id_map = HashMap::new();

		for (loc, desc, exc, times) in data {
			let entry = id_map.entry((loc, desc, exc)).or_insert(vec![]);

			if let Some(times) = times {
				entry.push(times);
			}
		}

		id_map
			.into_iter()
			.map(|((loc, desc, exc), times)| (loc, desc, exc, times))
			.collect()
	}

	/// Get a [`Location`] by its id and include its [`Translation`]s.
	///
	/// # Errors
	pub async fn get_by_id(
		loc_id: i32,
		conn: &DbConn,
	) -> Result<FullLocationData, Error> {
		let result = conn
			.interact(move |conn| {
				location::table
					.filter(location::id.eq(loc_id))
					.inner_join(
						description.on(location::description_id
							.eq(description.field(translation::id))),
					)
					.inner_join(excerpt.on(
						location::excerpt_id.eq(excerpt.field(translation::id)),
					))
					.left_outer_join(opening_time::table)
					.select((
						location::all_columns,
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
						opening_time::all_columns.nullable(),
					))
					.get_results(conn)
			})
			.await??;

		match Self::group_by_id(result).first() {
			Some(r) => Ok(r.clone()),
			None => Err(Error::NotFound(String::new())),
		}
	}

	/// Get all locations created by a given profile
	///
	/// # Errors
	pub async fn get_by_profile_id(
		profile_id: i32,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let locations = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				location
					.filter(created_by_id.eq(profile_id))
					.inner_join(description.on(
						description_id.eq(description.field(translation::id)),
					))
					.inner_join(
						excerpt
							.on(excerpt_id.eq(excerpt.field(translation::id))),
					)
					.left_outer_join(opening_time::table)
					.select((
						Location::as_select(),
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
						opening_time::all_columns.nullable(),
					))
					.load(conn)
			})
			.await??;

		Ok(Self::group_by_id(locations))
	}

	/// Get all the latlng positions of the locations.
	///
	/// # Errors
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
	///
	/// # Errors
	pub async fn delete_by_id(loc_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::location::dsl::*;

			diesel::delete(location.filter(id.eq(loc_id))).execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Approve a [`Location`] by its id and profile id.
	///
	/// # Errors
	pub async fn approve_by(
		loc_id: i32,
		profile_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
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

#[derive(
	Associations,
	Clone,
	Debug,
	Deserialize,
	Identifiable,
	Queryable,
	Selectable,
	Serialize,
)]
#[diesel(belongs_to(Location))]
#[diesel(table_name = location_image)]
pub struct LocationImage {
	pub id:          i32,
	pub location_id: i32,
	pub file_path:   String,
	pub uploaded_at: NaiveDateTime,
	pub uploaded_by: i32,
	pub approved_at: Option<NaiveDateTime>,
	pub approved_by: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = location_image)]
pub struct NewLocationImage {
	pub location_id: i32,
	pub file_path:   String,
	pub uploaded_by: i32,
}

impl NewLocationImage {
	/// Insert this list of [`NewLocationImage`]s into the database.
	///
	/// # Errors
	pub async fn bulk_insert(
		v: Vec<Self>,
		conn: &DbConn,
	) -> Result<Vec<LocationImage>, Error> {
		let images = conn
			.interact(move |conn| {
				use self::location_image::dsl::*;

				diesel::insert_into(location_image)
					.values(v)
					.returning(LocationImage::as_returning())
					.get_results(conn)
			})
			.await??;

		Ok(images)
	}
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
	///
	/// # Errors
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

#[derive(AsChangeset, Clone, Debug, Deserialize)]
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
