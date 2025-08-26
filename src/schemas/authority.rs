use authority::{
	Authority,
	AuthorityProfileUpdate,
	AuthorityUpdate,
	NewAuthority,
	NewAuthorityProfile,
};
use chrono::NaiveDateTime;
use primitives::PrimitiveAuthority;
use serde::{Deserialize, Serialize};

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

impl From<Authority> for AuthorityResponse {
	fn from(value: Authority) -> Self {
		Self {
			id:          value.authority.id,
			name:        value.authority.name,
			description: value.authority.description,
			created_at:  value.authority.created_at,
			created_by:  value.created_by.map(|p| p.map(Into::into)),
			updated_at:  value.authority.updated_at,
			updated_by:  value.updated_by.map(|p| p.map(Into::into)),
		}
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
	pub profile_id:  i32,
	pub permissions: i64,
}

impl CreateAuthorityMemberRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		authority_id: i32,
		added_by: i32,
	) -> NewAuthorityProfile {
		NewAuthorityProfile {
			authority_id,
			profile_id: self.profile_id,
			added_by,
			permissions: self.permissions,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAuthorityProfileRequest {
	pub permissions: i64,
}

impl UpdateAuthorityProfileRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> AuthorityProfileUpdate {
		AuthorityProfileUpdate { updated_by, permissions: self.permissions }
	}
}
