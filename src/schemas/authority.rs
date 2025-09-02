use authority::{
	Authority,
	AuthorityIncludes,
	AuthorityMemberUpdate,
	AuthorityUpdate,
	NewAuthority,
	NewAuthorityMember,
};
use chrono::NaiveDateTime;
use primitives::PrimitiveAuthority;
use serde::{Deserialize, Serialize};

use crate::schemas::BuildResponse;
use crate::schemas::profile::ProfileResponse;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityResponse {
	pub id:          i32,
	pub name:        String,
	pub description: Option<String>,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<Option<ProfileResponse>>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<Option<ProfileResponse>>,
}

impl BuildResponse<AuthorityResponse> for Authority {
	type Includes = AuthorityIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		_config: &crate::Config,
	) -> Result<AuthorityResponse, common::Error> {
		let created_by = self.created_by.map(Into::into);
		let updated_by = self.updated_by.map(Into::into);

		Ok(AuthorityResponse {
			id:          self.primitive.id,
			name:        self.primitive.name,
			description: self.primitive.description,
			created_at:  self.primitive.created_at,
			created_by:  if includes.created_by {
				Some(created_by)
			} else {
				None
			},
			updated_at:  self.primitive.updated_at,
			updated_by:  if includes.updated_by {
				Some(updated_by)
			} else {
				None
			},
		})
	}
}

impl From<PrimitiveAuthority> for AuthorityResponse {
	fn from(value: PrimitiveAuthority) -> Self {
		Self {
			id:          value.id,
			name:        value.name,
			description: value.description,
			created_at:  value.created_at,
			created_by:  None,
			updated_at:  value.updated_at,
			updated_by:  None,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAuthorityRequest {
	pub name:        String,
	pub description: Option<String>,
}

impl CreateAuthorityRequest {
	#[must_use]
	pub fn to_insertable(self, created_by: i32) -> NewAuthority {
		NewAuthority {
			name: self.name,
			description: self.description,
			created_by,
			institution_id: None,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAuthorityRequest {
	pub name:        Option<String>,
	pub description: Option<String>,
}

impl UpdateAuthorityRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> AuthorityUpdate {
		AuthorityUpdate {
			name: self.name,
			description: self.description,
			updated_by,
			institution_id: None,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAuthorityMemberRequest {
	pub profile_id:        i32,
	pub authority_role_id: Option<i32>,
}

impl CreateAuthorityMemberRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		authority_id: i32,
		added_by: i32,
	) -> NewAuthorityMember {
		NewAuthorityMember {
			authority_id,
			profile_id: self.profile_id,
			authority_role_id: self.authority_role_id,
			added_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityMemberUpdateRequest {
	pub authority_role_id: Option<i32>,
}

impl AuthorityMemberUpdateRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> AuthorityMemberUpdate {
		AuthorityMemberUpdate {
			authority_role_id: self.authority_role_id,
			updated_by,
		}
	}
}
