use std::hash::Hash;

use chrono::{NaiveDateTime, Utc};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel::{Identifiable, Queryable, Selectable};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{
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
	AuthorityPermissions,
	Image,
	NewImage,
	NewLocationImage,
	NewTranslation,
	PrimitiveAuthority,
	PrimitiveOpeningTime,
	PrimitiveTranslation,
	SimpleProfile,
	Tag,
};

mod filter;
mod member;

pub use filter::*;
pub use member::*;

pub type UnjoinedLocationData = (
	PrimitiveLocation,
	PrimitiveTranslation,
	PrimitiveTranslation,
	Option<PrimitiveAuthority>,
	Option<SimpleProfile>,
	Option<SimpleProfile>,
	Option<SimpleProfile>,
	Option<SimpleProfile>,
);

pub type LocationBackfill = (
	Vec<Location>,
	Vec<(i32, PrimitiveOpeningTime)>,
	Vec<(i32, Tag)>,
	Vec<(i32, Image)>,
);

pub type FullLocationData =
	(Location, (Vec<PrimitiveOpeningTime>, Vec<Tag>, Vec<Image>));

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct LocationIncludes {
	#[serde(default)]
	pub authority:   bool,
	#[serde(default)]
	pub approved_by: bool,
	#[serde(default)]
	pub rejected_by: bool,
	#[serde(default)]
	pub created_by:  bool,
	#[serde(default)]
	pub updated_by:  bool,
}

#[derive(Clone, Debug, Queryable, Serialize)]
#[diesel(table_name = location)]
#[diesel(check_for_backend(Pg))]
pub struct Location {
	pub location:    PrimitiveLocation,
	pub authority:   Option<Option<PrimitiveAuthority>>,
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

impl PrimitiveLocation {
	/// Get a [`PrimitiveLocation`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(l_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let location = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;

				location.find(l_id).select(Self::as_select()).first(conn)
			})
			.await??;

		Ok(location)
	}
}

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl Location {
	/// Build a query with all required (dynamic) joins to select a full
	/// location data tuple
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn joined_query(includes: LocationIncludes) -> _ {
		let inc_authority: bool = includes.authority;
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
				crate::schema::authority::table.on(inc_authority
					.into_sql::<Bool>()
					.and(
						crate::schema::location::authority_id
							.eq(crate::schema::authority::id.nullable()),
					)),
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

	/// Construct a full [`Location`] struct from the data returned by a
	/// joined query
	#[allow(clippy::many_single_char_names)]
	#[allow(clippy::too_many_arguments)]
	fn from_joined(
		includes: LocationIncludes,
		data: UnjoinedLocationData,
	) -> Self {
		Self {
			location:    data.0,
			description: data.1,
			excerpt:     data.2,
			authority:   if includes.authority { Some(data.3) } else { None },
			approved_by: if includes.approved_by { Some(data.4) } else { None },
			rejected_by: if includes.rejected_by { Some(data.5) } else { None },
			created_by:  if includes.created_by { Some(data.6) } else { None },
			updated_by:  if includes.updated_by { Some(data.7) } else { None },
		}
	}

	/// Group a locations and their related data together
	#[must_use]
	pub fn group(
		locs: Vec<Location>,
		times: &[(i32, PrimitiveOpeningTime)],
		tags: &[(i32, Tag)],
		imgs: &[(i32, Image)],
	) -> Vec<FullLocationData> {
		locs.into_par_iter()
			.map(|l| {
				let l_id = l.location.id;

				let times = times
					.iter()
					.filter(|(i, _)| *i == l_id)
					.map(|(_, d)| d.to_owned())
					.collect();
				let tags = tags
					.iter()
					.filter(|(i, _)| *i == l_id)
					.map(|(_, d)| d.to_owned())
					.collect();
				let imgs = imgs
					.iter()
					.filter(|(i, _)| *i == l_id)
					.map(|(_, d)| d.to_owned())
					.collect();

				(l, (times, tags, imgs))
			})
			.collect()
	}

	/// Get the permissions for a given user for this location
	#[instrument(skip(conn))]
	pub async fn get_profile_permissions(
		l_id: i32,
		p_id: i32,
		conn: &DbConn,
	) -> Result<
		(Option<AuthorityPermissions>, Option<LocationPermissions>),
		Error,
	> {
		let auth_perms: Option<i64> = conn
			.interact(move |conn| {
				use crate::schema::{authority, authority_profile};

				location::table
					.find(l_id)
					.left_outer_join(authority::table.on(
						location::authority_id.eq(authority::id.nullable()),
					))
					.left_outer_join(
						authority_profile::table.on(
							authority_profile::authority_id
								.eq(authority::id)
								.and(authority_profile::profile_id.eq(p_id)),
						),
					)
					.select(authority_profile::permissions.nullable())
					.get_result(conn)
			})
			.await??;

		let auth_perms =
			auth_perms.map(AuthorityPermissions::from_bits_truncate);

		let loc_perms: Option<i64> = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;
				use crate::schema::location_profile::dsl::*;

				location
					.find(l_id)
					.left_outer_join(
						location_profile
							.on(location_id.eq(id).and(profile_id.eq(p_id))),
					)
					.select(permissions.nullable())
					.get_result(conn)
			})
			.await??;

		let loc_perms = loc_perms.map(LocationPermissions::from_bits_truncate);

		Ok((auth_perms, loc_perms))
	}

	/// Get a [`Location`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		loc_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<FullLocationData, Error> {
		let query = Self::joined_query(includes);

		let location_data = conn
			.interact(move |conn| {
				use crate::schema::location::dsl::*;

				query
					.filter(id.eq(loc_id))
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
						<
							PrimitiveAuthority as Selectable<Pg>
						>
						::construct_selection().nullable(),
						approver.fields(simple_profile::all_columns).nullable(),
						rejecter.fields(simple_profile::all_columns).nullable(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let location = Self::from_joined(includes, location_data);
		let l_id = location.location.id;

		let (times, tags, imgs) = tokio::join!(
			PrimitiveOpeningTime::get_for_location(l_id, conn),
			Tag::get_for_location(l_id, conn),
			Image::get_for_location(l_id, conn),
		);

		let times = times?;
		let tags = tags?;
		let imgs = imgs?;

		Ok((location, (times, tags, imgs)))
	}

	/// Get all locations created by a given profile
	#[instrument(skip(conn))]
	pub async fn get_by_profile_id(
		profile_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let query = Self::joined_query(includes);

		let locations: Vec<_> = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(created_by.eq(profile_id))
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
						<
							PrimitiveAuthority as Selectable<Pg>
						>
						::construct_selection().nullable(),
						approver.fields(simple_profile::all_columns).nullable(),
						rejecter.fields(simple_profile::all_columns).nullable(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.load(conn)
			})
			.await??
			.into_iter()
			.map(|(l, d, e, y, a, r, c, u)| {
				Self::from_joined(includes, (l, d, e, y, a, r, c, u))
			})
			.collect();

		let l_ids: Vec<i32> = locations.iter().map(|l| l.location.id).collect();

		let (times, tags, imgs) = tokio::join!(
			PrimitiveOpeningTime::get_for_locations(l_ids.clone(), conn),
			Tag::get_for_locations(l_ids.clone(), conn),
			Image::get_for_locations(l_ids, conn),
		);

		let times = times?;
		let tags = tags?;
		let imgs = imgs?;

		Ok(Self::group(locations, &times, &tags, &imgs))
	}

	/// Get all simple locations belonging to an authority
	#[instrument(skip(conn))]
	pub async fn get_simple_by_authority_id(
		auth_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::joined_query(includes);

		let locations = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(authority_id.eq(auth_id))
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
						<
							PrimitiveAuthority as Selectable<Pg>
						>
						::construct_selection().nullable(),
						approver.fields(simple_profile::all_columns).nullable(),
						rejecter.fields(simple_profile::all_columns).nullable(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.load(conn)
			})
			.await??
			.into_iter()
			.map(|(l, d, e, y, a, r, c, u)| {
				Self::from_joined(includes, (l, d, e, y, a, r, c, u))
			})
			.collect();

		Ok(locations)
	}

	/// Get all locations belonging to an authority
	#[instrument(skip(conn))]
	pub async fn get_by_authority_id(
		auth_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let query = Self::joined_query(includes);

		let locations: Vec<_> = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(authority_id.eq(auth_id))
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
						<
							PrimitiveAuthority as Selectable<Pg>
						>
						::construct_selection().nullable(),
						approver.fields(simple_profile::all_columns).nullable(),
						rejecter.fields(simple_profile::all_columns).nullable(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.load(conn)
			})
			.await??
			.into_iter()
			.map(|(l, d, e, y, a, r, c, u)| {
				Self::from_joined(includes, (l, d, e, y, a, r, c, u))
			})
			.collect();

		let l_ids: Vec<i32> = locations.iter().map(|l| l.location.id).collect();

		let (times, tags, imgs) = tokio::join!(
			PrimitiveOpeningTime::get_for_locations(l_ids.clone(), conn),
			Tag::get_for_locations(l_ids.clone(), conn),
			Image::get_for_locations(l_ids, conn),
		);

		let times = times?;
		let tags = tags?;
		let imgs = imgs?;

		Ok(Self::group(locations, &times, &tags, &imgs))
	}

	/// Delete a [`Location`] by its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(loc_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::location::dsl::*;

			diesel::delete(location.filter(id.eq(loc_id))).execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Approve a [`Location`] by its id and profile id
	#[instrument(skip(conn))]
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

	/// Reject a [`Location`] by its id and profile id
	#[instrument(skip(conn))]
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
	#[instrument(skip(images, conn))]
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
	#[instrument(skip(conn))]
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
	#[instrument(skip(conn))]
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
