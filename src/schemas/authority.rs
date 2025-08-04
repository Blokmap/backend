use chrono::NaiveDateTime;
use models::{
	Authority,
	AuthorityProfileUpdate,
	AuthorityUpdate,
	Location,
	NewAuthority,
	NewAuthorityProfile,
	PrimitiveAuthority,
	PrimitiveProfile,
};
use serde::{Deserialize, Serialize};

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityResponse {
	pub id:          i32,
	pub name:        String,
	pub description: Option<String>,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<Option<PrimitiveProfile>>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<Option<PrimitiveProfile>>,
}

impl From<Authority> for AuthorityResponse {
	fn from(value: Authority) -> Self {
		Self {
			id:          value.authority.id,
			name:        value.authority.name,
			description: value.authority.description,
			created_at:  value.authority.created_at,
			created_by:  value.created_by,
			updated_at:  value.authority.updated_at,
			updated_by:  value.updated_by,
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

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullAuthorityResponse {
	pub id:          i32,
	pub name:        String,
	pub description: Option<String>,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<Option<PrimitiveProfile>>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<Option<PrimitiveProfile>>,
	pub members:     Vec<PrimitiveProfile>,
	pub locations:   Vec<Location>,
}

impl From<(Authority, Vec<PrimitiveProfile>, Vec<Location>)>
	for FullAuthorityResponse
{
	fn from(value: (Authority, Vec<PrimitiveProfile>, Vec<Location>)) -> Self {
		Self {
			id:          value.0.authority.id,
			name:        value.0.authority.name,
			description: value.0.authority.description,
			created_at:  value.0.authority.created_at,
			created_by:  value.0.created_by,
			updated_at:  value.0.authority.updated_at,
			updated_by:  value.0.updated_by,
			members:     value.1,
			locations:   value.2,
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
