use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Language, NewTranslation, Translation};

/// The data needed to make a new [`Translation`]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationRequest {
	pub language: Language,
	pub text:     String,
}

impl From<TranslationRequest> for NewTranslation {
	fn from(request: TranslationRequest) -> Self {
		let key = Uuid::new_v4();

		NewTranslation { key, language: request.language, text: request.text }
	}
}

/// The data returned when making a new [`Translation`]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResponse(pub Translation);

impl From<Translation> for TranslationResponse {
	fn from(value: Translation) -> Self { TranslationResponse(value) }
}

/// The data returned when making a list of new [`Translation`]s
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkTranslationsRequest(pub HashMap<Language, String>);

impl From<BulkTranslationsRequest> for Vec<NewTranslation> {
	fn from(request: BulkTranslationsRequest) -> Self {
		let BulkTranslationsRequest(translations) = request;
		let key = Uuid::new_v4();

		translations
			.into_iter()
			.map(|(language, text)| NewTranslation { key, language, text })
			.collect()
	}
}

/// The data returned when making a list of new [`Translation`]s
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkTranslationsResponse(pub HashMap<Language, Translation>);

impl From<Vec<Translation>> for BulkTranslationsResponse {
	fn from(value: Vec<Translation>) -> Self {
		let mut translations = HashMap::new();

		for translation in value {
			translations.insert(translation.language, translation);
		}

		BulkTranslationsResponse(translations)
	}
}
