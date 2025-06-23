use chrono::NaiveDateTime;
use models::{Authority, SimpleProfile};
use serde::{Deserialize, Serialize};

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
