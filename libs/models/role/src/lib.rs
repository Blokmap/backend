#[macro_use]
extern crate tracing;

use chrono::NaiveDateTime;
use db::{CreatorAlias, UpdaterAlias, creator, profile, updater};
use diesel::dsl::{AliasedFields, Nullable};
use diesel::prelude::*;
use primitives::PrimitiveProfile;
use serde::{Deserialize, Serialize};

mod authority;
mod institution;
mod location;

pub use authority::*;
pub use institution::*;
pub use location::*;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct RoleIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpaqueRole {
	pub id:          i32,
	pub name:        String,
	pub colour:      String,
	pub permissions: i64,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<PrimitiveProfile>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<PrimitiveProfile>,
}

impl From<LocationRole> for OpaqueRole {
	fn from(value: LocationRole) -> Self {
		Self {
			id:          value.primitive.id,
			name:        value.primitive.name,
			colour:      value.primitive.colour,
			permissions: value.primitive.permissions,
			created_at:  value.primitive.created_at,
			created_by:  value.created_by,
			updated_at:  value.primitive.updated_at,
			updated_by:  value.updated_by,
		}
	}
}

impl From<AuthorityRole> for OpaqueRole {
	fn from(value: AuthorityRole) -> Self {
		Self {
			id:          value.primitive.id,
			name:        value.primitive.name,
			colour:      value.primitive.colour,
			permissions: value.primitive.permissions,
			created_at:  value.primitive.created_at,
			created_by:  value.created_by,
			updated_at:  value.primitive.updated_at,
			updated_by:  value.updated_by,
		}
	}
}

impl From<InstitutionRole> for OpaqueRole {
	fn from(value: InstitutionRole) -> Self {
		Self {
			id:          value.primitive.id,
			name:        value.primitive.name,
			colour:      value.primitive.colour,
			permissions: value.primitive.permissions,
			created_at:  value.primitive.created_at,
			created_by:  value.created_by,
			updated_at:  value.primitive.updated_at,
			updated_by:  value.updated_by,
		}
	}
}

#[allow(non_camel_case_types)]
pub(crate) type created_by_fragment = Nullable<
	AliasedFields<CreatorAlias, <profile::table as Table>::AllColumns>,
>;
pub(crate) fn created_by_fragment() -> created_by_fragment {
	creator.fields(profile::all_columns).nullable()
}

#[allow(non_camel_case_types)]
pub(crate) type updated_by_fragment = Nullable<
	AliasedFields<UpdaterAlias, <profile::table as Table>::AllColumns>,
>;
pub(crate) fn updated_by_fragment() -> updated_by_fragment {
	updater.fields(profile::all_columns).nullable()
}
