use chrono::NaiveDateTime;
use models::{Institution, InstitutionCategory};
use serde::{Deserialize, Serialize};

use crate::schemas::profile::ProfileResponse;
use crate::schemas::ser_includes;
use crate::schemas::translation::TranslationResponse;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionResponse {
	pub id:               i32,
	pub name_translation: TranslationResponse,
	pub slug_translation: TranslationResponse,
	pub email:            Option<String>,
	pub phone_number:     Option<String>,
	pub street:           Option<String>,
	pub number:           Option<String>,
	pub zip:              Option<String>,
	pub city:             Option<String>,
	pub province:         Option<String>,
	pub country:          Option<String>,
	pub created_at:       NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub created_by:       Option<Option<ProfileResponse>>,
	pub updated_at:       NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub updated_by:       Option<Option<ProfileResponse>>,
	pub category:         InstitutionCategory,
	pub slug:             String,
}

impl From<Institution> for InstitutionResponse {
	fn from(value: Institution) -> Self {
		Self {
			id:               value.institution.id,
			name_translation: value.name.into(),
			slug_translation: value.slug.into(),
			email:            value.institution.email,
			phone_number:     value.institution.phone_number,
			street:           value.institution.street,
			number:           value.institution.number,
			zip:              value.institution.zip,
			city:             value.institution.city,
			province:         value.institution.province,
			country:          value.institution.country,
			created_at:       value.institution.created_at,
			created_by:       value.created_by.map(|p| p.map(Into::into)),
			updated_at:       value.institution.updated_at,
			updated_by:       value.updated_by.map(|p| p.map(Into::into)),
			category:         value.institution.category,
			slug:             value.institution.slug,
		}
	}
}
