use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DbConn;
use crate::error::Error;
use crate::schema::translation;

#[derive(Clone, DbEnum, Debug, Deserialize, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::Language"]
pub enum Language {
	Nl,
	En,
	Fr,
	De,
}

#[derive(Clone, Debug, Identifiable, Queryable, Selectable, Serialize)]
#[diesel(table_name = translation)]
pub struct Translation {
	pub id:         i32,
	pub language:   Language,
	pub key:        Uuid,
	pub text:       String,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewTranslations {
	pub key:          Option<Uuid>,
	pub translations: Vec<NewTranslation>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewTranslation {
	pub language: Language,
	pub key:      Option<Uuid>,
	pub text:     String,
}

#[derive(Debug, Deserialize, Clone, Insertable)]
#[diesel(table_name = translation)]
struct InsertableTranslation {
	pub language: Language,
	pub key:      Uuid,
	pub text:     String,
}

impl NewTranslations {
	pub async fn insert(self, conn: DbConn) -> Result<(Uuid, Vec<Translation>), Error> {
		// Unwrap is safe as invald UUIDs are caught by serde validation
		let key = self.key.unwrap_or_else(Uuid::new_v4);

		let insertables: Vec<InsertableTranslation> = self
			.translations
			.into_iter()
			.map(|t| InsertableTranslation { language: t.language, key, text: t.text })
			.collect();

		let translations = conn
			.interact(|conn| {
				conn.transaction(|conn| {
					use self::translation::dsl::*;

					diesel::insert_into(translation)
						.values(insertables)
						.returning(Translation::as_returning())
						.get_results(conn)
				})
			})
			.await??;

		Ok((key, translations))
	}
}

impl From<NewTranslation> for InsertableTranslation {
	fn from(value: NewTranslation) -> Self {
		// Unwrap is safe as invald UUIDs are caught by serde validation
		let key = value.key.unwrap_or_else(Uuid::new_v4);

		Self { language: value.language, key, text: value.text }
	}
}

impl NewTranslation {
	pub async fn insert(self, conn: DbConn) -> Result<Translation, Error> {
		let insertable_translation: InsertableTranslation = self.into();

		let new_translation = conn
			.interact(|conn| {
				use self::translation::dsl::*;

				diesel::insert_into(translation)
					.values(insertable_translation)
					.returning(Translation::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(new_translation)
	}
}

impl Translation {
	pub async fn get_all(conn: DbConn) -> Result<Vec<Self>, Error> {
		let translations = conn
			.interact(|conn| {
				use self::translation::dsl::*;

				translation.load(conn)
			})
			.await??;

		Ok(translations)
	}

	pub async fn get_by_key(query_key: Uuid, conn: DbConn) -> Result<Vec<Self>, Error> {
		let translations = conn
			.interact(move |conn| {
				use self::translation::dsl::*;

				translation.select(Translation::as_select()).filter(key.eq(query_key)).load(conn)
			})
			.await??;

		Ok(translations)
	}

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
}
