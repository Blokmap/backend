use serde::{Deserialize, Serialize};

use crate::models::{NewTranslation, Translation, UpdateTranslation};

/// The data needed to make a new [`Translation`].
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTranslationRequest {
	#[serde(flatten)]
	pub translation: NewTranslation,
}

/// The data needed to update a [`Translation`].
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTranslationRequest {
	#[serde(flatten)]
	pub translation: UpdateTranslation,
}

/// The data returned when making a new [`Translation`]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResponse {
	#[serde(flatten)]
	pub translation: Translation,
}

impl From<Translation> for TranslationResponse {
	fn from(translation: Translation) -> Self { Self { translation } }
}
