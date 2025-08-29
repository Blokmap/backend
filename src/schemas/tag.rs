use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tag::{NewTag, Tag, TagIncludes, TagUpdate};

use crate::schemas::BuildResponse;
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

impl BuildResponse<TagResponse> for Tag {
	type Includes = TagIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		_config: &crate::Config,
	) -> Result<TagResponse, common::Error> {
		let created_by = self.created_by.map(Into::into);
		let updated_by = self.updated_by.map(Into::into);

		Ok(TagResponse {
			id:         self.primitive.id,
			name:       self.name.into(),
			created_at: self.primitive.created_at,
			created_by: if includes.created_by {
				Some(created_by)
			} else {
				None
			},
			updated_at: self.primitive.updated_at,
			updated_by: if includes.updated_by {
				Some(updated_by)
			} else {
				None
			},
		})
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
