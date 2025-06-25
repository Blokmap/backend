use chrono::NaiveDateTime;
use models::{NewTag, PrimitiveTranslation, SimpleProfile, Tag, TagUpdate};
use serde::{Deserialize, Serialize};

use crate::schemas::translation::{
	CreateTranslationRequest,
	UpdateTranslationRequest,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagResponse {
	pub id:         i32,
	pub name:       PrimitiveTranslation,
	pub created_at: NaiveDateTime,
	pub created_by: Option<Option<SimpleProfile>>,
	pub updated_at: NaiveDateTime,
	pub updated_by: Option<Option<SimpleProfile>>,
}

impl From<Tag> for TagResponse {
	fn from(value: Tag) -> Self {
		Self {
			id:         value.tag.id,
			name:       value.name,
			created_at: value.tag.created_at,
			created_by: value.created_by,
			updated_at: value.tag.updated_at,
			updated_by: value.updated_by,
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
