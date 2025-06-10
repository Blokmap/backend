use std::collections::HashMap;
use std::hash::Hash;

use chrono::{NaiveDateTime, Utc};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel::{Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::{
	DescriptionAlias,
	ExcerptAlias,
	approver,
	creator,
	description,
	excerpt,
	location,
	opening_time,
	rejecter,
	simple_profile,
	translation,
	updater,
};
use crate::{
	Image,
	NewImage,
	NewLocationImage,
	NewTranslation,
	PaginationOptions,
	PrimitiveOpeningTime,
	PrimitiveTranslation,
	SimpleProfile,
};

mod filter;

pub use filter::*;

pub type LocationBackfill = (Location, Option<PrimitiveOpeningTime>);

pub type FullLocationData = (Location, Vec<PrimitiveOpeningTime>);

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct LocationIncludes {
	#[serde(default)]
	pub approved_by: bool,
	#[serde(default)]
	pub rejected_by: bool,
	#[serde(default)]
	pub created_by:  bool,
	#[serde(default)]
	pub updated_by:  bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = location)]
#[diesel(check_for_backend(Pg))]
pub struct Location {
	pub location:    PrimitiveLocation,
	// TODO: authorities
	// pub authority:              Option<i32>,
	pub description: PrimitiveTranslation,
	pub excerpt:     PrimitiveTranslation,
	pub approved_by: Option<Option<SimpleProfile>>,
	pub rejected_by: Option<Option<SimpleProfile>>,
	pub created_by:  Option<Option<SimpleProfile>>,
	pub updated_by:  Option<Option<SimpleProfile>>,
}

impl Hash for Location {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.location.id.hash(state);
	}
}

impl PartialEq for Location {
	fn eq(&self, other: &Self) -> bool { self.location.id == other.location.id }
}

impl Eq for Location {}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = location)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveLocation {
	pub id:                     i32,
	pub name:                   String,
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
	pub rejected_at:            Option<NaiveDateTime>,
	pub rejected_reason:        Option<String>,
	pub created_at:             NaiveDateTime,
	pub updated_at:             NaiveDateTime,
}

impl Location {
	fn group_by_id(data: Vec<LocationBackfill>) -> Vec<FullLocationData> {
		let mut id_map = HashMap::new();

		for (loc, times) in data {
			let entry = id_map.entry(loc).or_insert(vec![]);

			if let Some(times) = times {
				entry.push(times);
			}
		}

		id_map.into_iter().collect()
	}

	/// Get all [`Location`]s
	#[allow(clippy::too_many_lines)]
	pub async fn get_all(
		includes: LocationIncludes,
		p_opts: PaginationOptions,
		conn: &DbConn,
	) -> Result<(i64, Vec<FullLocationData>), Error> {
		let total: i64 = conn
			.interact(move |conn| {
				use diesel::dsl::count;

				use crate::schema::location::dsl::*;

				location.select(count(id)).first(conn)
			})
			.await??;

		let locations: Vec<LocationBackfill> = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;

				location
					.inner_join(
						description.on(description_id
							.eq(description.field(translation::id))),
					)
					.inner_join(
						excerpt.on(excerpt_id
							.eq(excerpt.field(translation::id))),
					)
					.left_outer_join(approver.on(
						includes.approved_by.into_sql::<Bool>().and(
							approved_by
								.eq(approver.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(rejecter.on(
						includes.rejected_by.into_sql::<Bool>().and(
							rejected_by
								.eq(rejecter.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							created_by
								.eq(creator.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							updated_by
								.eq(updater.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(opening_time::table)
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
					.limit(p_opts.limit())
					.offset(p_opts.offset())
					.load(conn)
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

	/// Get a [`Location`] by its id
	#[allow(clippy::too_many_lines)]
	pub async fn get_by_id(
		loc_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<FullLocationData, Error> {
		let locations: Vec<LocationBackfill> = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;

				location
					.filter(id.eq(loc_id))
					.inner_join(
						description.on(description_id
							.eq(description.field(translation::id))),
					)
					.inner_join(excerpt.on(
						excerpt_id.eq(excerpt.field(translation::id)),
					))
					.left_outer_join(approver.on(
						includes.approved_by.into_sql::<Bool>().and(
							approved_by
								.eq(approver.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(rejecter.on(
						includes.rejected_by.into_sql::<Bool>().and(
							rejected_by
								.eq(rejecter.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							created_by
								.eq(creator.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							updated_by
								.eq(updater.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(opening_time::table)
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

		match Self::group_by_id(locations).first() {
			Some(r) => Ok(r.clone()),
			None => Err(Error::NotFound(String::new())),
		}
	}

	/// Get all locations created by a given profile
	pub async fn get_by_profile_id(
		profile_id: i32,
		includes: LocationIncludes,
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
					.left_outer_join(approver.on(
						includes.approved_by.into_sql::<Bool>().and(
							approved_by
								.eq(approver.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(rejecter.on(
						includes.rejected_by.into_sql::<Bool>().and(
							rejected_by
								.eq(rejecter.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							created_by
								.eq(creator.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							updated_by
								.eq(updater.field(simple_profile::id)
								.nullable())
						)
					))
					.left_outer_join(opening_time::table)
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
					.load(conn)
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
					rejected_at.eq(None::<NaiveDateTime>),
					rejected_by.eq(None::<i32>),
					rejected_reason.eq(None::<String>),
				))
				.execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Reject a [`Location`] by its id and profile id.
	///
	/// # Errors
	pub async fn reject_by(
		loc_id: i32,
		profile_id: i32,
		reason: Option<String>,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::location::dsl::*;

			diesel::update(location.filter(id.eq(loc_id)))
				.set((
					approved_by.eq(None::<i32>),
					approved_at.eq(None::<NaiveDateTime>),
					rejected_at.eq(Utc::now().naive_utc()),
					rejected_by.eq(profile_id),
					rejected_reason.eq(reason),
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
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewLocation {
	pub name:                   String,
	pub authority_id:           Option<i32>,
	pub description:            NewTranslation,
	pub excerpt:                NewTranslation,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub min_reservation_length: Option<i32>,
	pub max_reservation_length: Option<i32>,
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
pub struct InsertableNewLocation {
	pub name:                   String,
	pub authority_id:           Option<i32>,
	pub description_id:         i32,
	pub excerpt_id:             i32,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub min_reservation_length: Option<i32>,
	pub max_reservation_length: Option<i32>,
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
	/// Create a new [`Location`]
	pub async fn insert(
		self,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<FullLocationData, Error> {
		let location = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use crate::schema::location::dsl::location;
					use crate::schema::translation::dsl::translation;

					let desc = diesel::insert_into(translation)
						.values(self.description)
						.returning(PrimitiveTranslation::as_returning())
						.get_result(conn)?;

					let exc = diesel::insert_into(translation)
						.values(self.excerpt)
						.returning(PrimitiveTranslation::as_returning())
						.get_result(conn)?;

					let new_location = InsertableNewLocation {
						name:                   self.name,
						authority_id:           self.authority_id,
						description_id:         desc.id,
						excerpt_id:             exc.id,
						seat_count:             self.seat_count,
						is_reservable:          self.is_reservable,
						reservation_block_size: self.reservation_block_size,
						max_reservation_length: self.max_reservation_length,
						min_reservation_length: self.min_reservation_length,
						street:                 self.street,
						number:                 self.number,
						zip:                    self.zip,
						city:                   self.city,
						country:                self.country,
						province:               self.province,
						latitude:               self.latitude,
						longitude:              self.longitude,
						created_by:             self.created_by,
					};

					let loc = diesel::insert_into(location)
						.values(new_location)
						.returning(PrimitiveLocation::as_returning())
						.get_result(conn)?;

					Ok(loc)
				})
			})
			.await??;

		let location = Location::get_by_id(location.id, includes, conn).await?;

		info!("inserted new location {location:?}");

		Ok(location)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize)]
#[diesel(table_name = crate::schema::location)]
pub struct LocationUpdate {
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
	pub updated_by:    i32,
}

impl LocationUpdate {
	/// Update this [`Location`] in the database.
	pub async fn apply_to(
		self,
		loc_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<FullLocationData, Error> {
		let location = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				diesel::update(location.filter(id.eq(loc_id)))
					.set(self)
					.returning(PrimitiveLocation::as_returning())
					.get_result(conn)
			})
			.await??;

		let location = Location::get_by_id(location.id, includes, conn).await?;

		info!("updated location {location:?}");

		Ok(location)
	}
}
