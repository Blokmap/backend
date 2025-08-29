#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate tracing;

use std::hash::Hash;

use ::image::{Image, OrderedImage};
use ::opening_time::{OpeningTime, OpeningTimeIncludes, TimeBoundsFilter};
use ::role::NewRole;
use ::tag::Tag;
use ::translation::NewTranslation;
use chrono::{NaiveDateTime, Utc};
use common::{DbConn, Error};
use db::{
	ApproverAlias,
	CreatorAlias,
	DescriptionAlias,
	ExcerptAlias,
	RejecterAlias,
	UpdaterAlias,
	approver,
	authority,
	creator,
	description,
	excerpt,
	location,
	location_member,
	opening_time,
	profile,
	rejecter,
	role,
	translation,
	updater,
};
use diesel::dsl::{AliasedFields, Nullable, sql};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Double};
use image::ImageIncludes;
use permissions::Permissions;
use primitives::{
	PrimitiveAuthority,
	PrimitiveLocation,
	PrimitiveProfile,
	PrimitiveTranslation,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use tag::TagIncludes;

mod filter;
mod member;

pub use filter::*;
pub use member::*;

pub type JoinedLocationData = (
	PrimitiveLocation,
	PrimitiveTranslation,
	PrimitiveTranslation,
	Option<PrimitiveAuthority>,
	Option<PrimitiveProfile>,
	Option<PrimitiveProfile>,
	Option<PrimitiveProfile>,
	Option<PrimitiveProfile>,
);

pub type FullLocationData =
	(Location, (Vec<OpeningTime>, Vec<Tag>, Vec<OrderedImage>));

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

#[serde_as]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
	#[serde_as(as = "DisplayFromStr")]
	pub center_lat: f64,
	#[serde_as(as = "DisplayFromStr")]
	pub center_lng: f64,
}

#[derive(Clone, Debug, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct Location {
	#[diesel(embed)]
	pub primitive:   PrimitiveLocation,
	#[diesel(embed)]
	pub authority:   Option<PrimitiveAuthority>,
	#[diesel(select_expression = description_fragment())]
	pub description: PrimitiveTranslation,
	#[diesel(select_expression = excerpt_fragment())]
	pub excerpt:     PrimitiveTranslation,
	#[diesel(select_expression = approved_by_fragment())]
	pub approved_by: Option<PrimitiveProfile>,
	#[diesel(select_expression = rejected_by_fragment())]
	pub rejected_by: Option<PrimitiveProfile>,
	#[diesel(select_expression = created_by_fragment())]
	pub created_by:  Option<PrimitiveProfile>,
	#[diesel(select_expression = updated_by_fragment())]
	pub updated_by:  Option<PrimitiveProfile>,
}

#[allow(non_camel_case_types)]
type description_fragment =
	AliasedFields<DescriptionAlias, <translation::table as Table>::AllColumns>;
fn description_fragment() -> description_fragment {
	description.fields(translation::all_columns)
}

#[allow(non_camel_case_types)]
type excerpt_fragment =
	AliasedFields<ExcerptAlias, <translation::table as Table>::AllColumns>;
fn excerpt_fragment() -> excerpt_fragment {
	excerpt.fields(translation::all_columns)
}

#[allow(non_camel_case_types)]
type approved_by_fragment = Nullable<
	AliasedFields<ApproverAlias, <profile::table as Table>::AllColumns>,
>;
fn approved_by_fragment() -> approved_by_fragment {
	approver.fields(profile::all_columns).nullable()
}

#[allow(non_camel_case_types)]
type rejected_by_fragment = Nullable<
	AliasedFields<RejecterAlias, <profile::table as Table>::AllColumns>,
>;
fn rejected_by_fragment() -> rejected_by_fragment {
	rejecter.fields(profile::all_columns).nullable()
}

#[allow(non_camel_case_types)]
type created_by_fragment = Nullable<
	AliasedFields<CreatorAlias, <profile::table as Table>::AllColumns>,
>;
fn created_by_fragment() -> created_by_fragment {
	creator.fields(profile::all_columns).nullable()
}

#[allow(non_camel_case_types)]
type updated_by_fragment = Nullable<
	AliasedFields<UpdaterAlias, <profile::table as Table>::AllColumns>,
>;
fn updated_by_fragment() -> updated_by_fragment {
	updater.fields(profile::all_columns).nullable()
}

impl Hash for Location {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.primitive.id.hash(state);
	}
}

impl PartialEq for Location {
	fn eq(&self, other: &Self) -> bool {
		self.primitive.id == other.primitive.id
	}
}

impl Eq for Location {}

impl Location {
	/// Build a query with all required (dynamic) joins to select a full
	/// location data tuple
	#[rustfmt::skip] // rustfmt hates me and i hate rustfmt
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: LocationIncludes) -> _ {
		let inc_authority: bool = includes.authority;
		let inc_approved_by: bool = includes.approved_by;
		let inc_rejected_by: bool = includes.rejected_by;
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		location::table
			.inner_join(description.on(
				location::description_id
					.eq(description.field(translation::id)),
			))
			.inner_join(excerpt.on(
				location::excerpt_id
					.eq(excerpt.field(translation::id))
			))
			.left_join(
				authority::table.on(
					inc_authority.into_sql::<Bool>()
					.and(location::authority_id.eq(authority::id.nullable()))
			))
			.left_join(
				approver.on(
					inc_approved_by.into_sql::<Bool>()
					.and(
						location::approved_by.
							eq(approver.field(profile::id).nullable()),
					)
			))
			.left_join(
				rejecter.on(
					inc_rejected_by.into_sql::<Bool>()
					.and(
						location::rejected_by
							.eq(rejecter.field(profile::id).nullable()),
					)
			))
			.left_join(
				creator.on(
					inc_created_by.into_sql::<Bool>()
					.and(
						location::created_by
							.eq(creator.field(profile::id).nullable())
					)
			))
			.left_join(
				updater.on(
					inc_updated_by.into_sql::<Bool>()
					.and(
						location::updated_by
							.eq(updater.field(profile::id).nullable())
					)
			))
	}

	/// Group a locations and their related data together
	#[must_use]
	pub fn group(
		locs: Vec<Location>,
		times: &[(i32, OpeningTime)],
		tags: &[(i32, Tag)],
		imgs: &[(i32, OrderedImage)],
	) -> Vec<FullLocationData> {
		locs.into_par_iter()
			.map(|l| {
				let l_id = l.primitive.id;

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

	/// Get a [`Location`] with no extra info by its id
	#[instrument(skip(conn))]
	pub async fn get_simple_by_id(
		loc_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::query(includes);

		let location = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(id.eq(loc_id))
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		Ok(location)
	}

	/// Get a [`Location`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		loc_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<FullLocationData, Error> {
		let query = Self::query(includes);

		let location = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(id.eq(loc_id))
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		let l_id = location.primitive.id;

		let (times, tags, imgs) = tokio::join!(
			OpeningTime::get_for_location(
				l_id,
				TimeBoundsFilter::default(),
				OpeningTimeIncludes::default(),
				conn
			),
			Tag::get_for_location(l_id, TagIncludes::default(), conn),
			Image::get_for_location(l_id, ImageIncludes::default(), conn),
		);

		let times = times?;
		let tags = tags?;
		let imgs = imgs?;

		Ok((location, (times, tags, imgs)))
	}

	/// Get a list of [`Location`]s given a list of IDs
	#[instrument(skip(conn))]
	pub async fn get_by_ids(
		loc_ids: Vec<i32>,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let query = Self::query(includes);

		let locations: Vec<Location> = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(id.eq_any(loc_ids))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		let l_ids: Vec<i32> =
			locations.iter().map(|l| l.primitive.id).collect();

		let (times, tags, imgs) = tokio::join!(
			OpeningTime::get_for_locations(
				l_ids.clone(),
				OpeningTimeIncludes::default(),
				conn
			),
			Tag::get_for_locations(l_ids.clone(), TagIncludes::default(), conn),
			Image::get_for_locations(l_ids, ImageIncludes::default(), conn),
		);

		let times = times?;
		let tags = tags?;
		let imgs = imgs?;

		Ok(Self::group(locations, &times, &tags, &imgs))
	}

	/// Get all locations created by a given profile
	#[instrument(skip(conn))]
	pub async fn get_by_profile_id(
		profile_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let query = Self::query(includes);

		let locations: Vec<_> = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(created_by.eq(profile_id))
					.select(Self::as_select())
					.load(conn)
			})
			.await??;

		let l_ids: Vec<i32> =
			locations.iter().map(|l| l.primitive.id).collect();

		let (times, tags, imgs) = tokio::join!(
			OpeningTime::get_for_locations(
				l_ids.clone(),
				OpeningTimeIncludes::default(),
				conn
			),
			Tag::get_for_locations(l_ids.clone(), TagIncludes::default(), conn),
			Image::get_for_locations(l_ids, ImageIncludes::default(), conn),
		);

		let times = times?;
		let tags = tags?;
		let imgs = imgs?;

		Ok(Self::group(locations, &times, &tags, &imgs))
	}

	/// Get the location nearest to the given point
	#[instrument(skip(conn))]
	pub async fn get_nearest(
		point: Point,
		conn: &DbConn,
	) -> Result<(i32, f64, f64), Error> {
		let loc_info: (i32, f64, f64) = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				location
					.filter(is_visible.eq(true))
					.order(
						sql::<Double>("sqrt(power(latitude - ")
							.bind::<Double, _>(point.center_lat)
							.sql(", 2) + power(longitude - ")
							.bind::<Double, _>(point.center_lng)
							.sql(",2))")
							.asc(),
					)
					.select((id, latitude, longitude))
					.first(conn)
			})
			.await??;

		Ok(loc_info)
	}

	/// Get all simple locations belonging to an authority
	#[instrument(skip(conn))]
	pub async fn get_simple_by_authority_id(
		auth_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let locations = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(authority_id.eq(auth_id))
					.select(Self::as_select())
					.load(conn)
			})
			.await??;

		Ok(locations)
	}

	/// Get all locations belonging to an authority
	#[instrument(skip(conn))]
	pub async fn get_by_authority_id(
		auth_id: i32,
		includes: LocationIncludes,
		conn: &DbConn,
	) -> Result<Vec<FullLocationData>, Error> {
		let query = Self::query(includes);

		let locations: Vec<_> = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				query
					.filter(authority_id.eq(auth_id))
					.left_outer_join(opening_time::table)
					.select(Self::as_select())
					.load(conn)
			})
			.await??;

		let l_ids: Vec<i32> =
			locations.iter().map(|l| l.primitive.id).collect();

		let (times, tags, imgs) = tokio::join!(
			OpeningTime::get_for_locations(
				l_ids.clone(),
				OpeningTimeIncludes::default(),
				conn
			),
			Tag::get_for_locations(l_ids.clone(), TagIncludes::default(), conn),
			Image::get_for_locations(l_ids, ImageIncludes::default(), conn),
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
					rejected_by.eq(None::<i32>),
					rejected_at.eq(None::<NaiveDateTime>),
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
					rejected_by.eq(profile_id),
					rejected_at.eq(Utc::now().naive_utc()),
					rejected_reason.eq(reason),
				))
				.execute(conn)
		})
		.await??;

		Ok(())
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
#[diesel(table_name = self::location)]
pub struct InsertableNewLocation {
	pub name:                   String,
	pub authority_id:           Option<i32>,
	pub description_id:         i32,
	pub excerpt_id:             i32,
	pub seat_count:             i32,
	pub is_reservable:          bool,
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
					use self::location::dsl::location;
					use self::translation::dsl::translation;

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
						max_reservation_length: self.max_reservation_length,
						is_visible:             self.is_visible,
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

					let new_role = NewRole {
						name:        "owner".into(),
						colour:      None,
						permissions: Permissions::LocAdministrator.bits(),
						created_by:  self.created_by,
					};

					let role_id = diesel::insert_into(role::table)
						.values(new_role)
						.returning(role::id)
						.get_result(conn)?;

					let member = NewLocationMember {
						location_id: loc.id,
						profile_id:  self.created_by,
						role_id:     Some(role_id),
						added_by:    self.created_by,
					};

					diesel::insert_into(location_member::table)
						.values(member)
						.execute(conn)?;

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
#[diesel(table_name = self::location)]
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
