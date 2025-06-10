use chrono::NaiveDateTime;
use models::{NewTranslation, SimpleProfile, Translation, TranslationUpdate};
use serde::{Deserialize, Serialize};

/// The data returned when making a new [`Translation`]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResponse {
	pub id:         i32,
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub created_at: NaiveDateTime,
	pub created_by: Option<Option<SimpleProfile>>,
	pub updated_at: NaiveDateTime,
	pub updated_by: Option<Option<SimpleProfile>>,
}

impl From<Translation> for TranslationResponse {
	fn from(value: Translation) -> Self {
		Self {
			id:         value.translation.id,
			nl:         value.translation.nl,
			en:         value.translation.en,
			fr:         value.translation.fr,
			de:         value.translation.de,
			created_at: value.translation.created_at,
			created_by: value.created_by,
			updated_at: value.translation.updated_at,
			updated_by: value.updated_by,
		}
	}
}

/// The data needed to make a new [`Translation`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTranslationRequest {
	pub nl: Option<String>,
	pub en: Option<String>,
	pub fr: Option<String>,
	pub de: Option<String>,
}

impl CreateTranslationRequest {
	#[must_use]
	pub fn to_insertable(self, created_by: i32) -> NewTranslation {
		NewTranslation {
			nl: self.nl,
			en: self.en,
			fr: self.fr,
			de: self.de,
			created_by,
		}
	}
}

/// The data needed to update a [`Translation`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTranslationRequest {
	pub nl: Option<String>,
	pub en: Option<String>,
	pub fr: Option<String>,
	pub de: Option<String>,
}

impl UpdateTranslationRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> TranslationUpdate {
		TranslationUpdate {
			nl: self.nl,
			en: self.en,
			fr: self.fr,
			de: self.de,
			updated_by,
		}
	}
}
