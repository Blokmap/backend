use chrono::NaiveDateTime;
use db::InstitutionCategory;
use models::{
	Institution,
	InstitutionProfileUpdate,
	NewInstitution,
	NewInstitutionProfile,
};
use serde::{Deserialize, Serialize};

use crate::schemas::authority::{AuthorityResponse, CreateAuthorityRequest};
use crate::schemas::profile::ProfileResponse;
use crate::schemas::ser_includes;
use crate::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
};

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionResponse {
	pub id:               i32,
	pub name_translation: TranslationResponse,
	pub email:            Option<String>,
	pub phone_number:     Option<String>,
	pub street:           Option<String>,
	pub number:           Option<String>,
	pub zip:              Option<String>,
	pub city:             Option<String>,
	pub province:         Option<String>,
	pub country:          Option<String>,
	pub created_at:       NaiveDateTime,
	pub created_by:       Option<ProfileResponse>,
	pub updated_at:       NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub updated_by:       Option<Option<ProfileResponse>>,
	pub category:         InstitutionCategory,
	pub slug:             String,
	pub authority:        Option<AuthorityResponse>,
}

impl From<Institution> for InstitutionResponse {
	fn from(value: Institution) -> Self {
		Self {
			id:               value.institution.id,
			name_translation: value.name.into(),
			email:            value.institution.email,
			phone_number:     value.institution.phone_number,
			street:           value.institution.street,
			number:           value.institution.number,
			zip:              value.institution.zip,
			city:             value.institution.city,
			province:         value.institution.province,
			country:          value.institution.country,
			created_at:       value.institution.created_at,
			created_by:       value.created_by.map(Into::into),
			updated_at:       value.institution.updated_at,
			updated_by:       value.updated_by.map(|p| p.map(Into::into)),
			category:         value.institution.category,
			slug:             value.institution.slug,
			authority:        None,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstitutionRequest {
	pub name_translation: CreateTranslationRequest,
	pub email:            Option<String>,
	pub phone_number:     Option<String>,
	pub street:           Option<String>,
	pub number:           Option<String>,
	pub zip:              Option<String>,
	pub city:             Option<String>,
	pub province:         Option<String>,
	pub country:          Option<String>,
	pub category:         InstitutionCategory,
	pub slug:             String,
	pub authority:        Option<CreateAuthorityRequest>,
}

impl CreateInstitutionRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		created_by: i32,
	) -> (NewInstitution, Option<CreateAuthorityRequest>) {
		(
			NewInstitution {
				name_translation: self
					.name_translation
					.to_insertable(created_by),
				email: self.email,
				phone_number: self.phone_number,
				street: self.street,
				number: self.number,
				zip: self.zip,
				city: self.city,
				province: self.province,
				country: self.country,
				created_by,
				category: self.category,
				slug: self.slug,
			},
			self.authority,
		)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstitutionMemberRequest {
	pub profile_id:  i32,
	pub permissions: i64,
}

impl CreateInstitutionMemberRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		institution_id: i32,
		added_by: i32,
	) -> NewInstitutionProfile {
		NewInstitutionProfile {
			institution_id,
			profile_id: self.profile_id,
			added_by,
			permissions: self.permissions,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstitutionProfileRequest {
	pub permissions: i64,
}

impl UpdateInstitutionProfileRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> InstitutionProfileUpdate {
		InstitutionProfileUpdate { updated_by, permissions: self.permissions }
	}
}
