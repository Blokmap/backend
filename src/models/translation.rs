use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::translation;
use crate::{DbConn, Error};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize, Hash)]
pub enum Language {
	Nl,
	En,
	Fr,
	De,
}

impl From<Language> for String {
	fn from(language: Language) -> Self {
		match language {
			Language::Nl => "nl".to_string(),
			Language::En => "en".to_string(),
			Language::Fr => "fr".to_string(),
			Language::De => "de".to_string(),
		}
	}
}

#[derive(
	Clone, Debug, Deserialize, Serialize, Identifiable, Queryable, Selectable,
)]
#[diesel(table_name = translation)]
#[serde(rename_all = "camelCase")]
pub struct Translation {
	pub id:         i32,
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

/// Translation service functions.
impl Translation {
	// /// Get a list of all [`Translation`]s
	// pub(crate) async fn get_all(conn: DbConn) -> Result<Vec<Self>, Error> {
	// 	let translations = conn
	// 		.interact(|conn| {
	// 			use self::translation::dsl::*;
	//
	// 			translation.load(conn)
	// 		})
	// 		.await??;
	//
	// 	Ok(translations)
	// }

	/// Attempt to get a single [`Translation`] given its [key](Uuid) and
	/// [language](Language)
	pub(crate) async fn get_by_id(
		query_id: i32,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let translation = conn
			.interact(move |conn| {
				use self::translation::dsl::*;

				translation
					.select(Translation::as_select())
					.filter(id.eq(query_id))
					.get_result(conn)
			})
			.await??;

		Ok(translation)
	}

	/// Delete a single [`Translation`] given its [id](i32).
	pub(crate) async fn delete_by_id(
		query_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::translation::dsl::*;

			diesel::delete(translation.filter(id.eq(query_id))).execute(conn)
		})
		.await??;

		Ok(())
	}
}

#[derive(Debug, Deserialize, Serialize, Clone, Insertable)]
#[diesel(table_name = translation)]
pub struct NewTranslation {
	pub nl: Option<String>,
	pub en: Option<String>,
	pub fr: Option<String>,
	pub de: Option<String>,
}

impl NewTranslation {
	/// Insert this [`NewTranslation`].
	pub(crate) async fn insert(
		self,
		conn: &DbConn,
	) -> Result<Translation, Error> {
		let new_translation = conn
			.interact(|conn| {
				use self::translation::dsl::*;

				diesel::insert_into(translation)
					.values(self)
					.returning(Translation::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(new_translation)
	}
}

#[derive(Debug, Deserialize, Default, Serialize, AsChangeset)]
#[serde(default, rename_all = "camelCase")]
#[diesel(table_name = translation)]
pub struct UpdateTranslation {
	pub nl: Option<String>,
	pub en: Option<String>,
	pub fr: Option<String>,
	pub de: Option<String>,
}

impl UpdateTranslation {
	/// Update this [`UpdateTranslation`].
	pub(crate) async fn update(
		self,
		query_id: i32,
		conn: &DbConn,
	) -> Result<Translation, Error> {
		let updated_translation = conn
			.interact(move |conn| {
				use self::translation::dsl::*;

				diesel::update(translation.filter(id.eq(query_id)))
					.set(&self)
					.returning(Translation::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(updated_translation)
	}
}
