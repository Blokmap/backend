use chrono::NaiveDateTime;
use models::{Authority, Location, SimpleProfile};
use serde::{Deserialize, Serialize};

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityResponse {
	pub id:          i32,
	pub name:        String,
	pub description: Option<String>,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<Option<SimpleProfile>>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<Option<SimpleProfile>>,
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

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullAuthorityResponse {
	pub id:          i32,
	pub name:        String,
	pub description: Option<String>,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<Option<SimpleProfile>>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<Option<SimpleProfile>>,
	pub members:     Vec<SimpleProfile>,
	pub locations:   Vec<Location>,
}

impl From<(Authority, Vec<SimpleProfile>, Vec<Location>)>
	for FullAuthorityResponse
{
	fn from(value: (Authority, Vec<SimpleProfile>, Vec<Location>)) -> Self {
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
