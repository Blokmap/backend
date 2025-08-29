use chrono::NaiveDateTime;
use role::{NewRole, Role, RoleIncludes, RoleUpdate};
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

impl BuildResponse<RoleResponse> for Role {
	type Includes = RoleIncludes;

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
			colour:      self.primitive.colour,
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
pub struct CreateRoleRequest {
	pub name:        String,
	pub colour:      Option<String>,
	pub permissions: i64,
}

impl CreateRoleRequest {
	#[must_use]
	pub fn to_insertable(self, created_by: i32) -> NewRole {
		NewRole {
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
	pub fn to_insertable(self, updated_by: i32) -> RoleUpdate {
		RoleUpdate {
			name: self.name,
			colour: self.colour,
			permissions: self.permissions,
			updated_by,
		}
	}
}
