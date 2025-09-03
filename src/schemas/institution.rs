use chrono::NaiveDateTime;
use db::InstitutionCategory;
use institution::{
	Institution,
	InstitutionIncludes,
	InstitutionMemberUpdate,
	NewInstitution,
	NewInstitutionMember,
};
use serde::{Deserialize, Serialize};

use crate::schemas::authority::{AuthorityResponse, CreateAuthorityRequest};
use crate::schemas::profile::ProfileResponse;
use crate::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
};
use crate::schemas::{BuildResponse, ser_includes};

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

impl BuildResponse<InstitutionResponse> for Institution {
	type Includes = InstitutionIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		_config: &crate::Config,
	) -> Result<InstitutionResponse, common::Error> {
		let created_by = self.created_by.map(Into::into);
		let updated_by = self.updated_by.map(Into::into);

		Ok(InstitutionResponse {
			id:               self.primitive.id,
			name_translation: self.name.into(),
			email:            self.primitive.email,
			phone_number:     self.primitive.phone_number,
			street:           self.primitive.street,
			number:           self.primitive.number,
			zip:              self.primitive.zip,
			city:             self.primitive.city,
			province:         self.primitive.province,
			country:          self.primitive.country,
			created_at:       self.primitive.created_at,
			created_by:       if includes.created_by {
				created_by
			} else {
				None
			},
			updated_at:       self.primitive.updated_at,
			updated_by:       if includes.updated_by {
				Some(updated_by)
			} else {
				None
			},
			category:         self.primitive.category,
			slug:             self.primitive.slug,
			authority:        None,
		})
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
	pub profile_id:          i32,
	pub institution_role_id: Option<i32>,
}

impl CreateInstitutionMemberRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		institution_id: i32,
		added_by: i32,
	) -> NewInstitutionMember {
		NewInstitutionMember {
			institution_id,
			profile_id: self.profile_id,
			institution_role_id: self.institution_role_id,
			added_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionMemberUpdateRequest {
	pub institution_role_id: Option<i32>,
}

impl InstitutionMemberUpdateRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> InstitutionMemberUpdate {
		InstitutionMemberUpdate {
			institution_role_id: self.institution_role_id,
			updated_by,
		}
	}
}
