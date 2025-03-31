use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::translation;
use crate::{DbConn, Error};

#[derive(Clone, DbEnum, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::Language"]
pub enum Language {
	Nl,
	En,
	Fr,
	De,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = translation)]
pub struct Translation {
	pub id:         i32,
	pub language:   Language,
	pub key:        Uuid,
	pub text:       String,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Clone, Insertable)]
#[diesel(table_name = translation)]
pub struct InsertableTranslation {
	pub language: Language,
	pub key:      Uuid,
	pub text:     String,
}

impl InsertableTranslation {
	/// Insert this [`InsertableTranslation`]
	///
	/// # Errors
	/// Errors if interacting with the database fails
	pub async fn insert(self, conn: DbConn) -> Result<Translation, Error> {
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
	///
	/// # Errors
	/// Errors if interacting with the database fails
	pub async fn get_by_key_and_language(
		query_key: Uuid,
		query_language: Language,
		conn: DbConn,
	) -> Result<Self, Error> {
		let translation = conn
			.interact(move |conn| {
				use self::translation::dsl::*;

				translation
					.select(Translation::as_select())
					.filter(key.eq(query_key))
					.filter(language.eq(query_language))
					.get_result(conn)
			})
			.await??;

		Ok(translation)
	}

	/// Get a list of all [`Translation`]s that match the given [key](Uuid)
	///
	/// # Errors
	/// Errors if interacting with the database fails
	pub async fn get_by_key(
		query_key: Uuid,
		conn: DbConn,
	) -> Result<Vec<Self>, Error> {
		let translations = conn
			.interact(move |conn| {
				use self::translation::dsl::*;

				translation
					.select(Translation::as_select())
					.filter(key.eq(query_key))
					.load(conn)
			})
			.await??;

		Ok(translations)
	}

	/// Delete a single [`Translation`] given its [key](Uuid) and
	/// [language](Language)
	///
	/// # Errors
	/// Errors if interacting with the database fails
	pub async fn delete_by_key_and_language(
		query_key: Uuid,
		query_language: Language,
		conn: DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::translation::dsl::*;

			diesel::delete(
				translation
					.filter(key.eq(query_key))
					.filter(language.eq(query_language)),
			)
			.execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Delete all [`Translation`]s that match the given [key](Uuid)
	///
	/// # Errors
	/// Errors if interacting with the database fails
	pub async fn delete_by_key(
		query_key: Uuid,
		conn: DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::translation::dsl::*;

			diesel::delete(translation.filter(key.eq(query_key))).execute(conn)
		})
		.await??;

		Ok(())
	}
}
