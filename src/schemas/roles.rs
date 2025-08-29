use chrono::NaiveDateTime;
use location::{
	LocationRole,
	LocationRoleIncludes,
	LocationRoleUpdate,
	NewLocationRole,
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
	pub permissions: i64,
	pub created_at:  NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub created_by:  Option<Option<ProfileResponse>>,
	pub updated_at:  NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub updated_by:  Option<Option<ProfileResponse>>,
}

impl BuildResponse<RoleResponse> for LocationRole {
	type Includes = LocationRoleIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		_config: &Config,
	) -> Result<RoleResponse, common::Error> {
		let created_by = self.created_by.map(Into::into);
		let updated_by = self.updated_by.map(Into::into);

		Ok(RoleResponse {
			id:          self.primitive.id,
			name:        self.primitive.name,
			permissions: self.primitive.permissions,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRoleRequest {
	pub name:        String,
	pub permissions: i64,
}

impl CreateLocationRoleRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		location_id: i32,
		created_by: i32,
	) -> NewLocationRole {
		NewLocationRole {
			location_id,
			name: self.name,
			permissions: self.permissions,
			created_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationRoleRequest {
	pub name:        Option<String>,
	pub permissions: Option<i64>,
}

impl UpdateLocationRoleRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> LocationRoleUpdate {
		LocationRoleUpdate {
			name: self.name,
			permissions: self.permissions,
			updated_by,
		}
	}
}
