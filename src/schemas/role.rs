use chrono::NaiveDateTime;
use role::{
	AuthorityRole,
	AuthorityRoleUpdate,
	InstitutionRole,
	InstitutionRoleUpdate,
	LocationRole,
	LocationRoleUpdate,
	NewAuthorityRole,
	NewInstitutionRole,
	NewLocationRole,
	OpaqueRole,
	RoleIncludes,
};
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::schemas::profile::ProfileResponse;
use crate::schemas::{BuildResponse, ser_includes};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleResponse {
	pub id:          i32,
	pub name:        String,
	pub colour:      String,
	pub permissions: i64,
	pub created_at:  NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub created_by:  Option<Option<ProfileResponse>>,
	pub updated_at:  NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub updated_by:  Option<Option<ProfileResponse>>,
}

impl BuildResponse<RoleResponse> for OpaqueRole {
	type Includes = RoleIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		_config: &Config,
	) -> Result<RoleResponse, common::Error> {
		let created_by = self.created_by.map(Into::into);
		let updated_by = self.updated_by.map(Into::into);

		Ok(RoleResponse {
			id:          self.id,
			name:        self.name,
			colour:      self.colour,
			permissions: self.permissions,
			created_at:  self.created_at,
			created_by:  if includes.created_by {
				Some(created_by)
			} else {
				None
			},
			updated_at:  self.updated_at,
			updated_by:  if includes.updated_by {
				Some(updated_by)
			} else {
				None
			},
		})
	}
}

impl BuildResponse<RoleResponse> for LocationRole {
	type Includes = RoleIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		config: &Config,
	) -> Result<RoleResponse, common::Error> {
		OpaqueRole::from(self).build_response(includes, config)
	}
}

impl BuildResponse<RoleResponse> for AuthorityRole {
	type Includes = RoleIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		config: &Config,
	) -> Result<RoleResponse, common::Error> {
		OpaqueRole::from(self).build_response(includes, config)
	}
}

impl BuildResponse<RoleResponse> for InstitutionRole {
	type Includes = RoleIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		config: &Config,
	) -> Result<RoleResponse, common::Error> {
		OpaqueRole::from(self).build_response(includes, config)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
	pub name:        String,
	pub colour:      Option<String>,
	pub permissions: i64,
}

impl CreateRoleRequest {
	#[must_use]
	pub fn to_insertable_for_location(
		self,
		location_id: i32,
		created_by: i32,
	) -> NewLocationRole {
		NewLocationRole {
			location_id,
			name: self.name,
			colour: self.colour,
			permissions: self.permissions,
			created_by,
		}
	}

	#[must_use]
	pub fn to_insertable_for_authority(
		self,
		authority_id: i32,
		created_by: i32,
	) -> NewAuthorityRole {
		NewAuthorityRole {
			authority_id,
			name: self.name,
			colour: self.colour,
			permissions: self.permissions,
			created_by,
		}
	}

	#[must_use]
	pub fn to_insertable_for_institution(
		self,
		institution_id: i32,
		created_by: i32,
	) -> NewInstitutionRole {
		NewInstitutionRole {
			institution_id,
			name: self.name,
			colour: self.colour,
			permissions: self.permissions,
			created_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
	pub name:        Option<String>,
	pub colour:      Option<String>,
	pub permissions: Option<i64>,
}

impl UpdateRoleRequest {
	#[must_use]
	pub fn to_insertable_for_location(
		self,
		updated_by: i32,
	) -> LocationRoleUpdate {
		LocationRoleUpdate {
			name: self.name,
			colour: self.colour,
			permissions: self.permissions,
			updated_by,
		}
	}

	#[must_use]
	pub fn to_insertable_for_authority(
		self,
		updated_by: i32,
	) -> AuthorityRoleUpdate {
		AuthorityRoleUpdate {
			name: self.name,
			colour: self.colour,
			permissions: self.permissions,
			updated_by,
		}
	}

	#[must_use]
	pub fn to_insertable_for_institution(
		self,
		updated_by: i32,
	) -> InstitutionRoleUpdate {
		InstitutionRoleUpdate {
			name: self.name,
			colour: self.colour,
			permissions: self.permissions,
			updated_by,
		}
	}
}
