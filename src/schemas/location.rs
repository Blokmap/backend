use serde::{Deserialize, Serialize};

use super::translation::TranslationResponse;
use crate::models::{Location, NewLocation, UpdateLocation};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRequest {
	#[serde(flatten)]
	pub location: NewLocation,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationRequest {
    #[serde(flatten)]
    pub location: UpdateLocation,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocationResponse {
	pub id:          i32,
	pub name:        String,
	pub description: Option<TranslationResponse>,
	pub excerpt:     Option<TranslationResponse>,
}

impl From<Location> for LocationResponse {
	fn from(location: Location) -> Self {
		Self {
			id:          location.id,
			name:        location.name,
			description: None,
			excerpt:     None,
		}
	}
}
