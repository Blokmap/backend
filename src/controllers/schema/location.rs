use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::translation::{BulkTranslationsRequest, BulkTranslationsResponse};
use crate::models::{Location, NewLocation, Translation};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocationRequest {
	pub name:          String,
	pub description:   BulkTranslationsRequest,
	pub excerpt:       BulkTranslationsRequest,
	pub seat_count:    i32,
	pub is_reservable: bool,
	pub is_visible:    bool,
	pub street:        String,
	pub number:        String,
	pub city:          String,
	pub zip:           String,
	pub province:      String,
	pub latitude:      f64,
	pub longitude:     f64,
}

impl LocationRequest {
	/// Convert the request into a [`NewLocation`].
	pub fn to_new_location(
		self,
		excerpt_key: Uuid,
		description_key: Uuid,
	) -> NewLocation {
		let (idx, idy) = Location::get_cell_idx(self.latitude, self.longitude);

		NewLocation {
			name: self.name,
			description_key,
			excerpt_key,
			seat_count: self.seat_count,
			is_reservable: self.is_reservable,
			is_visible: self.is_visible,
			street: self.street,
			number: self.number,
			city: self.city,
			zip: self.zip,
			province: self.province,
			latitude: self.latitude,
			longitude: self.longitude,
            cell_idx: idx,
            cell_idy: idy,
		}
	}
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LocationResponse {
	pub name:        String,
	pub description: BulkTranslationsResponse,
	pub excerpt:     BulkTranslationsResponse,
}

impl LocationResponse {
	pub fn from_location(
		location: Location,
		description: Vec<Translation>,
		excerpt: Vec<Translation>,
	) -> Self {
		LocationResponse {
			name:        location.name,
			description: description.into(),
			excerpt:     excerpt.into(),
		}
	}
}
