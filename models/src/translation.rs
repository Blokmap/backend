use std::hash::Hash;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::translation;

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

#[derive(Clone, Debug, Deserialize, Serialize, Queryable, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = translation)]
#[diesel(check_for_backend(Pg))]
pub struct Translation {
	pub id:         i32,
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub created_at: NaiveDateTime,
	pub created_by: Option<i32>,
	pub updated_at: NaiveDateTime,
	pub updated_by: Option<i32>,
}

impl Hash for Translation {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state) }
}

impl PartialEq for Translation {
	fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for Translation {}

#[derive(Clone, Debug, Deserialize, Serialize, Queryable, Selectable)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = translation)]
#[diesel(check_for_backend(Pg))]
pub struct SimpleTranslation {
	pub id: i32,
	pub nl: Option<String>,
	pub en: Option<String>,
	pub fr: Option<String>,
	pub de: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Insertable)]
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
	///
	/// # Errors
	/// Errors if interacting with the database fails
	pub async fn insert(self, conn: &DbConn) -> Result<Translation, Error> {
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

	/// Insert a list of [`InsertableTranslation`]s in a single transaction
	///
	/// # Errors
	/// Errors if interacting with the database fails
	pub async fn bulk_insert(
		translations: Vec<Self>,
		conn: DbConn,
	) -> Result<Vec<Translation>, Error> {
		let translations = conn
			.interact(|conn| {
				conn.transaction(|conn| {
					use self::translation::dsl::*;

					diesel::insert_into(translation)
						.values(translations)
						.returning(Translation::as_returning())
						.get_results(conn)
				})
			})
			.await??;

		Ok(translations)
	}
}

impl Translation {
	/// Check if a [`Translation`] with a given id exists
	///
	/// # Errors
	pub async fn exists(query_id: i32, conn: &DbConn) -> Result<bool, Error> {
		let exists = conn
			.interact(move |conn| {
				use self::translation::dsl::*;
				diesel::select(diesel::dsl::exists(
					translation.filter(id.eq(query_id)),
				))
				.get_result(conn)
			})
			.await??;

		Ok(exists)
	}

	/// Attempt to get a single [`Translation`] given is id.
	///
	/// # Errors
	/// Errors if interacting with the database fails
	/// Errors if the [`Translation`] does not exist
	pub async fn get_by_id(
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
	///
	/// # Errors
	/// Errors if interacting with the database fails
	/// Errors if the [`Translation`] does not exist
	/// Errors if the [`Translation`] cannot be deleted
	pub async fn delete_by_id(
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

#[derive(Clone, Debug, Deserialize, Default, Serialize, AsChangeset)]
#[serde(default, rename_all = "camelCase")]
#[diesel(table_name = translation)]
pub struct UpdateTranslation {
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub updated_by: i32,
}

impl UpdateTranslation {
	/// Update this [`UpdateTranslation`].
	///
	/// # Errors
	/// Errors if interacting with the database fails
	/// Errors if the [`UpdateTranslation`] does not exist
	pub async fn apply_to(
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
