use serde::{Deserialize, Serialize};

use super::translation::TranslationResponse;
use crate::models::{Location, NewLocation, Translation, UpdateLocation};

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
	#[serde(flatten)]
	pub location:    Location,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<TranslationResponse>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub excerpt:     Option<TranslationResponse>,
}

impl From<Location> for LocationResponse {
	fn from(location: Location) -> Self {
		Self { location, description: None, excerpt: None }
	}
}

impl From<(Location, Translation, Translation)> for LocationResponse {
	fn from(
		(location, description, excerpt): (Location, Translation, Translation),
	) -> Self {
		Self {
			location,
			description: Some(description.into()),
			excerpt: Some(excerpt.into()),
		}
	}
}
