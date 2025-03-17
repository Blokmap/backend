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
	pub name:          String,
	pub seat_count:    i32,
	pub is_reservable: bool,
	pub is_visible:    bool,
	pub street:        String,
	pub number:        String,
	pub zip:           String,
	pub city:          String,
	pub province:      String,
	pub coords:        (f64, f64),
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description:   Option<TranslationResponse>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub excerpt:       Option<TranslationResponse>,
}

impl From<Location> for LocationResponse {
	fn from(location: Location) -> Self {
		Self {
			name:          location.name,
			seat_count:    location.seat_count,
			is_reservable: location.is_reservable,
			is_visible:    location.is_visible,
			street:        location.street,
			number:        location.number,
			zip:           location.zip,
			city:          location.city,
			province:      location.province,
			coords:        (location.latitude, location.longitude),
			description:   None,
			excerpt:       None,
		}
	}
}

impl From<(Location, Translation, Translation)> for LocationResponse {
	fn from(
		(location, description, excerpt): (Location, Translation, Translation),
	) -> Self {
		Self {
			name:          location.name,
			seat_count:    location.seat_count,
			is_reservable: location.is_reservable,
			is_visible:    location.is_visible,
			street:        location.street,
			number:        location.number,
			zip:           location.zip,
			city:          location.city,
			province:      location.province,
			coords:        (location.latitude, location.longitude),
			description:   Some(description.into()),
			excerpt:       Some(excerpt.into()),
		}
	}
}
