use chrono::NaiveDateTime;
use models::{NewTag, Tag, TagUpdate};
use serde::{Deserialize, Serialize};

use crate::schemas::profile::ProfileResponse;
use crate::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
	UpdateTranslationRequest,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagResponse {
	pub id:         i32,
	pub name:       TranslationResponse,
	pub created_at: NaiveDateTime,
	pub created_by: Option<Option<ProfileResponse>>,
	pub updated_at: NaiveDateTime,
	pub updated_by: Option<Option<ProfileResponse>>,
}

impl From<Tag> for TagResponse {
	fn from(value: Tag) -> Self {
		Self {
			id:         value.tag.id,
			name:       value.name.into(),
			created_at: value.tag.created_at,
			created_by: value.created_by.map(|p| p.map(Into::into)),
			updated_at: value.tag.updated_at,
			updated_by: value.updated_by.map(|p| p.map(Into::into)),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLocationTagsRequest {
	pub tags: Vec<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTagRequest {
	pub name: CreateTranslationRequest,
}

impl CreateTagRequest {
	#[must_use]
	pub fn to_insertable(self, created_by: i32) -> NewTag {
		let name = self.name.to_insertable(created_by);

		NewTag { name, created_by }
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagRequest {
	pub name: UpdateTranslationRequest,
}

impl UpdateTagRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> TagUpdate {
		let name = self.name.to_insertable(updated_by);

		TagUpdate { name, updated_by }
	}
}
