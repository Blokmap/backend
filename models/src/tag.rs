use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};

use crate::schema::{
	creator,
	location_tag,
	simple_profile,
	tag,
	translation,
	updater,
};
use crate::{
	NewTranslation,
	PrimitiveTranslation,
	SimpleProfile,
	TranslationUpdate,
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct TagIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(Pg))]
pub struct Tag {
	pub tag:        PrimitiveTag,
	pub name:       PrimitiveTranslation,
	pub created_by: Option<Option<SimpleProfile>>,
	pub updated_by: Option<Option<SimpleProfile>>,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveTag {
	pub id:         i32,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

impl Tag {
	/// Get a single [`Tag`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		tag_id: i32,
		includes: TagIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let tag: (
			PrimitiveTag,
			PrimitiveTranslation,
			Option<SimpleProfile>,
			Option<SimpleProfile>,
		) = conn
			.interact(move |conn| {
				use crate::schema::tag::dsl::*;

				tag.inner_join(
					translation::table
						.on(name_translation_id.eq(translation::id)),
				)
				.left_outer_join(
					creator.on(includes.created_by.into_sql::<Bool>().and(
						created_by
							.eq(creator.field(simple_profile::id).nullable()),
					)),
				)
				.left_outer_join(
					updater.on(includes.updated_by.into_sql::<Bool>().and(
						updated_by
							.eq(updater.field(simple_profile::id).nullable()),
					)),
				)
				.filter(id.eq(tag_id))
				.select((
					PrimitiveTag::as_select(),
					PrimitiveTranslation::as_select(),
					creator.fields(simple_profile::all_columns).nullable(),
					updater.fields(simple_profile::all_columns).nullable(),
				))
				.get_result(conn)
			})
			.await??;

		let tag = Self {
			tag:        tag.0,
			name:       tag.1,
			created_by: if includes.created_by { Some(tag.2) } else { None },
			updated_by: if includes.updated_by { Some(tag.3) } else { None },
		};

		Ok(tag)
	}

	/// Get all [`Tag`]s from the database, optionally including related
	/// profiles.
	#[instrument(skip(conn))]
	pub async fn get_all(
		includes: TagIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let tags = conn
			.interact(move |c| {
				tag::table
					.inner_join(
						translation::table
							.on(tag::name_translation_id.eq(translation::id)),
					)
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							tag::created_by.eq(
								creator.field(simple_profile::id).nullable(),
							),
						),
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							tag::updated_by.eq(
								updater.field(simple_profile::id).nullable(),
							),
						),
					))
					.select((
						PrimitiveTag::as_select(),
						PrimitiveTranslation::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.load(c)
			})
			.await??
			.into_iter()
			.map(|(tag, tr, cr, up)| {
				Tag {
					tag,
					name: tr,
					created_by: if includes.created_by {
						Some(cr)
					} else {
						None
					},
					updated_by: if includes.updated_by {
						Some(up)
					} else {
						None
					},
				}
			})
			.collect();

		Ok(tags)
	}

	/// Delete a [`Tag`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(tag_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::schema::tag::dsl::*;

			diesel::delete(tag.find(tag_id)).execute(conn)
		})
		.await??;

		info!("deleted tag with id {tag_id}");

		Ok(())
	}

	/// Get all tags for a location with the given id
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		l_id: i32,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let tags = conn
			.interact(move |conn| {
				use crate::schema::location;
				use crate::schema::location_tag::dsl::*;
				use crate::schema::tag::dsl::*;

				location::table
					.find(l_id)
					.inner_join(location_tag.on(location_id.eq(location::id)))
					.inner_join(tag.on(tag_id.eq(id)))
					.inner_join(
						translation::table
							.on(name_translation_id.eq(translation::id)),
					)
					.select((
						PrimitiveTag::as_select(),
						PrimitiveTranslation::as_select(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(tag, name)| {
				Tag { tag, name, created_by: None, updated_by: None }
			})
			.collect();

		Ok(tags)
	}

	/// Get all tags for a list of locations
	#[instrument(skip(conn))]
	pub async fn get_for_locations(
		l_ids: Vec<i32>,
		conn: &DbConn,
	) -> Result<Vec<(i32, Self)>, Error> {
		let tags = conn
			.interact(move |conn| {
				use crate::schema::location;
				use crate::schema::location_tag::dsl::*;
				use crate::schema::tag::dsl::*;

				location::table
					.filter(location::id.eq_any(l_ids))
					.inner_join(location_tag.on(location_id.eq(location::id)))
					.inner_join(tag.on(tag_id.eq(id)))
					.inner_join(
						translation::table
							.on(name_translation_id.eq(translation::id)),
					)
					.select((
						location::id,
						PrimitiveTag::as_select(),
						PrimitiveTranslation::as_select(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(id, tag, name)| {
				(id, Tag { tag, name, created_by: None, updated_by: None })
			})
			.collect();

		Ok(tags)
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = location_tag)]
#[diesel(check_for_backend(Pg))]
pub struct NewLocationTag {
	pub tag_id:      i32,
	pub location_id: i32,
}

impl Tag {
	/// Set a list of location-tag crossovers
	///
	/// This removes the previous list of location tags for this location
	#[instrument(skip(conn))]
	pub async fn bulk_set(
		l_id: i32,
		t_ids: Vec<i32>,
		conn: &DbConn,
	) -> Result<(), Error> {
		let new_tags: Vec<_> = t_ids
			.into_iter()
			.map(|tag_id| NewLocationTag { tag_id, location_id: l_id })
			.collect();

		conn.interact(move |conn| {
			conn.transaction(|conn| {
				use crate::schema::location_tag::dsl::*;

				diesel::delete(location_tag.filter(location_id.eq(l_id)))
					.execute(conn)?;

				diesel::insert_into(location_tag).values(new_tags).execute(conn)
			})
		})
		.await??;

		Ok(())
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewTag {
	pub name:       NewTranslation,
	pub created_by: i32,
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(Pg))]
struct InsertableNewTag {
	name_translation_id: i32,
	created_by:          i32,
}

impl NewTag {
	/// Insert this [`NewTag`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		includes: TagIncludes,
		conn: &DbConn,
	) -> Result<Tag, Error> {
		let tag = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use crate::schema::tag::dsl::tag;
					use crate::schema::translation::dsl::translation;

					let name_translation = diesel::insert_into(translation)
						.values(self.name)
						.returning(PrimitiveTranslation::as_returning())
						.get_result(conn)?;

					let new_tag = InsertableNewTag {
						name_translation_id: name_translation.id,
						created_by:          self.created_by,
					};

					let new_tag = diesel::insert_into(tag)
						.values(new_tag)
						.returning(PrimitiveTag::as_returning())
						.get_result(conn)?;

					Ok(new_tag)
				})
			})
			.await??;

		let tag = Tag::get_by_id(tag.id, includes, conn).await?;

		info!("created tag {tag:?}");

		Ok(tag)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TagUpdate {
	pub name:       TranslationUpdate,
	pub updated_by: i32,
}

#[derive(AsChangeset, Clone, Debug, Deserialize, Serialize)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(Pg))]
struct InsertableTagUpdate {
	updated_by: i32,
}

impl TagUpdate {
	/// Apply this update to the [`Tag`] with the given id
	pub async fn apply_to(
		self,
		tag_id: i32,
		includes: TagIncludes,
		conn: &DbConn,
	) -> Result<Tag, Error> {
		conn.interact(move |conn| {
			conn.transaction::<_, Error, _>(|conn| {
				use crate::schema::{tag, translation};

				let tag_update =
					InsertableTagUpdate { updated_by: self.updated_by };

				let name_translation_id: i32 =
					diesel::update(tag::table.find(tag_id))
						.set(tag_update)
						.returning(tag::name_translation_id)
						.get_result(conn)?;

				diesel::update(translation::table.find(name_translation_id))
					.set(self.name)
					.execute(conn)?;

				Ok(())
			})
		})
		.await??;

		let tag = Tag::get_by_id(tag_id, includes, conn).await?;

		info!("updated tag {tag:?}");

		Ok(tag)
	}
}
