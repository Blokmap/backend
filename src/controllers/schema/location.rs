use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::translation::BulkTranslationsResponse;
use crate::models::{Location, NewLocation};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocationRequest {
	pub name:            String,
	pub description_key: Uuid,
	pub excerpt_key:     Uuid,
	pub seat_count:      i32,
	pub is_reservable:   bool,
	pub is_visible:      bool,
	pub street:          String,
	pub number:          String,
	pub city:            String,
	pub zip:             String,
	pub province:        String,
	pub latitude:        f64,
	pub longitude:       f64,
}

impl From<LocationRequest> for NewLocation {
	fn from(request: LocationRequest) -> Self {
		NewLocation {
			name:            request.name,
			description_key: request.description_key,
			excerpt_key:     request.excerpt_key,
			seat_count:      request.seat_count,
			is_reservable:   request.is_reservable,
			is_visible:      request.is_visible,
			street:          request.street,
			number:          request.number,
			city:            request.city,
			zip:             request.zip,
			province:        request.province,
			latitude:        request.latitude,
			longitude:       request.longitude,
		}
	}
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocationResponse {
	pub id:          i32,
	pub name:        String,
	pub description: Option<BulkTranslationsResponse>,
	pub excerpt:     Option<BulkTranslationsResponse>,
}

impl From<Location> for LocationResponse {
	fn from(location: Location) -> Self {
		LocationResponse {
			id:          location.id,
			name:        location.name,
			description: None,
			excerpt:     None,
		}
	}
}
