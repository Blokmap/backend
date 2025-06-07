use std::collections::HashMap;
use std::hash::Hash;

use chrono::{NaiveDateTime, Utc};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::{Identifiable, Queryable, Selectable};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::{OpeningTime, Translation};
use crate::schema::{location, opening_time, profile, translation};
use crate::{
	Image,
	NewImage,
	NewLocationImage,
	NewTranslation,
	PaginationOptions,
	Profile,
};

mod filter;

pub use filter::*;

diesel::alias!(
	translation as description: DescriptionAlias,
	translation as excerpt: ExcerptAlias,
	profile as approver: ApproverAlias,
	profile as creater: CreaterAlias,
	profile as updater: UpdaterAlias,
);

pub type LocationBackfill = (
	Location,
	Translation,
	Translation,
	Option<OpeningTime>,
	Option<Profile>,
	Option<Profile>,
	Option<Profile>,
);

pub type FullLocationData = (
	Location,
	Translation,
	Translation,
	Vec<OpeningTime>,
	Option<Profile>,
	Option<Profile>,
	Option<Profile>,
);

#[derive(
	Clone, Debug, Deserialize, Serialize, Identifiable, Queryable, Selectable,
)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = location)]
#[diesel(check_for_backend(Pg))]
pub struct Location {
	pub id:                     i32,
	pub name:                   String,
	pub authority_id:           Option<i32>,
	pub description_id:         i32,
	pub excerpt_id:             i32,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub min_reservation_length: Option<i32>,
	pub max_reservation_length: Option<i32>,
	pub is_visible:             bool,
	pub street:                 String,
	pub number:                 String,
	pub zip:                    String,
	pub city:                   String,
	pub province:               String,
	pub country:                String,
	pub latitude:               f64,
	pub longitude:              f64,
	pub approved_at:            Option<NaiveDateTime>,
	pub approved_by:            Option<i32>,
	pub rejected_at:            Option<NaiveDateTime>,
	pub rejected_by:            Option<i32>,
	pub rejected_reason:        Option<String>,
	pub created_at:             NaiveDateTime,
	pub created_by:             Option<i32>,
	pub updated_at:             NaiveDateTime,
	pub updated_by:             Option<i32>,
}

impl Hash for Location {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state) }
}

impl PartialEq for Location {
	fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for Location {}

impl Location {
	fn group_by_id(data: Vec<LocationBackfill>) -> Vec<FullLocationData> {
		let mut id_map = HashMap::new();

		for (loc, desc, exc, times, aprv, crt, updt) in data {
			let entry = id_map
				.entry((loc, desc, exc, aprv, crt, updt))
				.or_insert(vec![]);

			if let Some(times) = times {
				entry.push(times);
			}
		}

		id_map
			.into_par_iter()
			.map(|((loc, desc, exc, aprv, crt, updt), times)| {
				(loc, desc, exc, times, aprv, crt, updt)
			})
			.collect()
	}

	/// Get all [`Location`]s
	///
	/// # Errors
	pub async fn get_all(
		p_opts: PaginationOptions,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let locations = conn
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
					.left_outer_join(
						approver.on(location::approved_by
							.eq(approver.field(profile::id).nullable())),
					)
					.left_outer_join(
						creater.on(location::created_by
							.eq(creater.field(profile::id).nullable())),
					)
					.left_outer_join(
						updater.on(location::updated_by
							.eq(updater.field(profile::id).nullable())),
					)
					.select((
						location::all_columns,
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
						opening_time::all_columns.nullable(),
						approver.fields(profile::all_columns).nullable(),
						creater.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.order(location::id)
					.limit(p_opts.limit())
					.offset(p_opts.offset())
					.load(conn)
			})
			.await??;

		Ok(Self::group_by_id(locations))
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
					.left_outer_join(
						approver.on(location::approved_by
							.eq(approver.field(profile::id).nullable())),
					)
					.left_outer_join(
						creater.on(location::created_by
							.eq(creater.field(profile::id).nullable())),
					)
					.left_outer_join(
						updater.on(location::updated_by
							.eq(updater.field(profile::id).nullable())),
					)
					.select((
						location::all_columns,
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
						opening_time::all_columns.nullable(),
						approver.fields(profile::all_columns).nullable(),
						creater.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
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
					.filter(created_by.eq(profile_id))
					.inner_join(description.on(
						description_id.eq(description.field(translation::id)),
					))
					.inner_join(
						excerpt
							.on(excerpt_id.eq(excerpt.field(translation::id))),
					)
					.left_outer_join(opening_time::table)
					.left_outer_join(approver.on(
						approved_by.eq(approver.field(profile::id).nullable()),
					))
					.left_outer_join(creater.on(
						created_by.eq(creater.field(profile::id).nullable()),
					))
					.left_outer_join(updater.on(
						updated_by.eq(updater.field(profile::id).nullable()),
					))
					.select((
						Location::as_select(),
						description.fields(translation::all_columns),
						excerpt.fields(translation::all_columns),
						opening_time::all_columns.nullable(),
						approver.fields(profile::all_columns).nullable(),
						creater.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
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
					approved_by.eq(profile_id),
					approved_at.eq(Utc::now().naive_utc()),
				))
				.execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Bulk insert a list of [`NewImage`]s for a specific [`Location`]
	///
	/// # Errors
	pub async fn insert_images(
		loc_id: i32,
		images: Vec<NewImage>,
		conn: &DbConn,
	) -> Result<Vec<Image>, Error> {
		let inserted_images = conn
			.interact(move |conn| {
				conn.transaction::<Vec<Image>, Error, _>(|conn| {
					use crate::schema::image::dsl::*;
					use crate::schema::location_image::dsl::*;

					let images = diesel::insert_into(image)
						.values(images)
						.returning(Image::as_returning())
						.get_results(conn)?;

					let location_images = images
						.iter()
						.map(|i| {
							NewLocationImage {
								location_id: loc_id,
								image_id:    i.id,
							}
						})
						.collect::<Vec<_>>();

					diesel::insert_into(location_image)
						.values(location_images)
						.execute(conn)?;

					Ok(images)
				})
			})
			.await??;

		Ok(inserted_images)
	}

	/// Create a new [`Location`] with its corresponding [`Translation`]s given
	/// a compound data struct
	///
	/// # Errors
	pub async fn new(
		loc_data: StubNewLocation,
		desc_data: NewTranslation,
		exc_data: NewTranslation,
		conn: &DbConn,
	) -> Result<(Self, Translation, Translation), Error> {
		let records = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use crate::schema::location::dsl::location;
					use crate::schema::translation::dsl::translation;

					let desc = diesel::insert_into(translation)
						.values(desc_data)
						.returning(Translation::as_returning())
						.get_result(conn)?;

					let exc = diesel::insert_into(translation)
						.values(exc_data)
						.returning(Translation::as_returning())
						.get_result(conn)?;

					let new_location = NewLocation {
						name:                   loc_data.name,
						description_id:         desc.id,
						excerpt_id:             exc.id,
						seat_count:             loc_data.seat_count,
						is_reservable:          loc_data.is_reservable,
						reservation_block_size: loc_data.reservation_block_size,
						is_visible:             loc_data.is_visible,
						street:                 loc_data.street,
						number:                 loc_data.number,
						zip:                    loc_data.zip,
						city:                   loc_data.city,
						country:                loc_data.country,
						province:               loc_data.province,
						latitude:               loc_data.latitude,
						longitude:              loc_data.longitude,
						created_by:             loc_data.created_by,
					};

					let loc = diesel::insert_into(location)
						.values(new_location)
						.returning(Location::as_returning())
						.get_result(conn)?;

					Ok((loc, desc, exc))
				})
			})
			.await??;

		Ok(records)
	}
}
#[derive(Clone, Debug, Deserialize)]
pub struct StubNewLocation {
	pub name:                   String,
	pub authority_id:           Option<i32>,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub min_reservation_length: Option<i32>,
	pub max_reservation_length: Option<i32>,
	pub is_visible:             bool,
	pub street:                 String,
	pub number:                 String,
	pub zip:                    String,
	pub city:                   String,
	pub country:                String,
	pub province:               String,
	pub latitude:               f64,
	pub longitude:              f64,
	pub created_by:             i32,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::location)]
pub struct NewLocation {
	pub name:                   String,
	pub description_id:         i32,
	pub excerpt_id:             i32,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub is_visible:             bool,
	pub street:                 String,
	pub number:                 String,
	pub zip:                    String,
	pub city:                   String,
	pub country:                String,
	pub province:               String,
	pub latitude:               f64,
	pub longitude:              f64,
	pub created_by:             i32,
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
	///
	/// # Errors
	/// Fails if interacting with the database fails
	pub async fn update(
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
