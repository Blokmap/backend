use serde::{Deserialize, Serialize};

use super::opening_time::OpeningTimeResponse;
use super::translation::TranslationResponse;
use crate::models::{Location, OpeningTime, Translation, UpdateLocation};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRequest {
	pub name:           String,
	pub description_id: i32,
	pub excerpt_id:     i32,
	pub seat_count:     i32,
	pub is_reservable:  bool,
	pub is_visible:     bool,
	pub street:         String,
	pub number:         String,
	pub zip:            String,
	pub city:           String,
	pub province:       String,
	pub latitude:       f64,
	pub longitude:      f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationRequest {
	#[serde(flatten)]
	pub location: UpdateLocation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationResponse {
	pub id:            i32,
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
	pub description:   Option<TranslationResponse>,
	pub excerpt:       Option<TranslationResponse>,
	pub image_paths:   Vec<String>,
	pub opening_times: Vec<OpeningTimeResponse>,
}

impl From<Location> for LocationResponse {
	fn from(location: Location) -> Self {
		Self {
			id:            location.id,
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
			image_paths:   vec![],
			opening_times: vec![],
		}
	}
}

impl From<(Location, Translation, Translation, Vec<OpeningTime>)>
	for LocationResponse
{
	fn from(
		(location, description, excerpt, opening_times): (
			Location,
			Translation,
			Translation,
			Vec<OpeningTime>,
		),
	) -> Self {
		Self {
			id:            location.id,
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
			image_paths: vec![],
			opening_times: opening_times.into_iter().map(Into::into).collect(),
		}
	}
}
