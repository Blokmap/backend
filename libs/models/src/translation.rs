use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};

use crate::PrimitiveProfile;
use crate::db::{creator, profile, translation, updater};

pub type JoinedTranslationData =
	(PrimitiveTranslation, Option<PrimitiveProfile>, Option<PrimitiveProfile>);

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct TranslationIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = translation)]
#[diesel(check_for_backend(Pg))]
pub struct Translation {
	pub translation: PrimitiveTranslation,
	pub created_by:  Option<Option<PrimitiveProfile>>,
	pub updated_by:  Option<Option<PrimitiveProfile>>,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = translation)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveTranslation {
	pub id:         i32,
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl Translation {
	/// Build a query with all required (dynamic) joins to select a full
	/// translation data tuple
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn joined_query(includes: TranslationIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		translation::table
			.left_outer_join(
				creator.on(inc_created_by.into_sql::<Bool>().and(
					translation::created_by
						.eq(creator.field(profile::id).nullable()),
				)),
			)
			.left_outer_join(
				updater.on(inc_updated_by.into_sql::<Bool>().and(
					translation::updated_by
						.eq(updater.field(profile::id).nullable()),
				)),
			)
	}

	/// Construct a full [`Translation`] struct from the data returned by a
	/// joined query
	#[allow(clippy::many_single_char_names)]
	#[allow(clippy::too_many_arguments)]
	fn from_joined(
		includes: TranslationIncludes,
		data: JoinedTranslationData,
	) -> Self {
		Self {
			translation: data.0,
			created_by:  if includes.created_by { Some(data.1) } else { None },
			updated_by:  if includes.updated_by { Some(data.2) } else { None },
		}
	}

	/// Attempt to get a single [`Translation`] given its id.
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		tr_id: i32,
		includes: TranslationIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query(includes);

		let translation = conn
			.interact(move |conn| {
				query
					.filter(translation::id.eq(tr_id))
					.select((
						PrimitiveTranslation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let translation = Self::from_joined(includes, translation);

		Ok(translation)
	}

	/// Delete a single [`Translation`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(tr_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::translation::dsl::*;

			diesel::delete(translation.find(tr_id)).execute(conn)
		})
		.await??;

		info!("deleted translation with id {tr_id}");

		Ok(())
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = translation)]
pub struct NewTranslation {
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub created_by: i32,
}

impl NewTranslation {
	/// Insert this [`NewTranslation`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		includes: TranslationIncludes,
		conn: &DbConn,
	) -> Result<Translation, Error> {
		let translation = conn
			.interact(|conn| {
				use self::translation::dsl::*;

				diesel::insert_into(translation)
					.values(self)
					.returning(PrimitiveTranslation::as_returning())
					.get_result(conn)
			})
			.await??;

		let translation =
			Translation::get_by_id(translation.id, includes, conn).await?;

		info!("created translation {translation:?}");

		Ok(translation)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize, Serialize)]
#[diesel(table_name = translation)]
pub struct TranslationUpdate {
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub updated_by: i32,
}

impl TranslationUpdate {
	/// Apply this update to the [`Translation`] with the given id
	pub async fn apply_to(
		self,
		tr_id: i32,
		includes: TranslationIncludes,
		conn: &DbConn,
	) -> Result<Translation, Error> {
		conn.interact(move |conn| {
			use self::translation::dsl::*;

			diesel::update(translation.find(tr_id)).set(self).execute(conn)
		})
		.await??;

		let translation = Translation::get_by_id(tr_id, includes, conn).await?;

		info!("updated translation {translation:?}");

		Ok(translation)
	}
}
