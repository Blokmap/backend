use chrono::NaiveDateTime;
use models::{
	Location,
	OpeningTime,
	Profile,
	StubNewLocation,
	Translation,
	UpdateLocation,
};
use serde::{Deserialize, Serialize};

use super::opening_time::OpeningTimeResponse;
use super::translation::TranslationResponse;
use crate::schemas::authority::AuthorityResponse;
use crate::schemas::image::ImageResponse;
use crate::schemas::profile::ProfileResponse;
use crate::schemas::tag::TagResponse;
use crate::schemas::translation::CreateTranslationRequest;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRequest {
	pub location:    LocationData,
	pub description: CreateTranslationRequest,
	pub excerpt:     CreateTranslationRequest,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationData {
	pub name:                   String,
	pub authority_id:           Option<i32>,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub min_reservation_length: Option<i32>,
	pub max_reservation_length: Option<i32>,
	pub is_visible:             bool,
	pub street:                 String,
	pub number:                 String,
	pub zip:                    String,
	pub city:                   String,
	pub province:               String,
	pub country:                String,
	pub latitude:               f64,
	pub longitude:              f64,
}

impl LocationData {
	#[must_use]
	pub fn to_insertable(self, created_by: i32) -> StubNewLocation {
		StubNewLocation {
			name: self.name,
			authority_id: self.authority_id,
			seat_count: self.seat_count,
			is_reservable: self.is_reservable,
			reservation_block_size: self.reservation_block_size,
			min_reservation_length: self.min_reservation_length,
			max_reservation_length: self.max_reservation_length,
			is_visible: self.is_visible,
			street: self.street,
			number: self.number,
			zip: self.zip,
			city: self.city,
			province: self.province,
			country: self.country,
			latitude: self.latitude,
			longitude: self.longitude,
			created_by,
		}
	}
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
	pub id:                     i32,
	pub name:                   String,
	pub authority:              Option<AuthorityResponse>,
	pub description:            Option<TranslationResponse>,
	pub excerpt:                Option<TranslationResponse>,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub min_reservation_length: Option<i32>,
	pub max_reservation_length: Option<i32>,
	pub is_visible:             bool,
	pub street:                 String,
	pub number:                 String,
	pub zip:                    String,
	pub city:                   String,
	pub province:               String,
	pub country:                String,
	pub latitude:               f64,
	pub longitude:              f64,
	pub images:                 Vec<ImageResponse>,
	pub opening_times:          Vec<OpeningTimeResponse>,
	pub tags:                   Vec<TagResponse>,
	pub approved_at:            Option<NaiveDateTime>,
	pub approved_by:            Option<ProfileResponse>,
	pub created_at:             NaiveDateTime,
	pub created_by:             Option<ProfileResponse>,
	pub updated_at:             NaiveDateTime,
	pub updated_by:             Option<ProfileResponse>,
}

impl From<Location> for LocationResponse {
	fn from(location: Location) -> Self {
		Self {
			id:                     location.id,
			name:                   location.name,
			authority:              None,
			description:            None,
			excerpt:                None,
			seat_count:             location.seat_count,
			is_reservable:          location.is_reservable,
			reservation_block_size: location.reservation_block_size,
			min_reservation_length: location.min_reservation_length,
			max_reservation_length: location.max_reservation_length,
			is_visible:             location.is_visible,
			street:                 location.street,
			number:                 location.number,
			zip:                    location.zip,
			city:                   location.city,
			province:               location.province,
			country:                location.country,
			latitude:               location.latitude,
			longitude:              location.longitude,
			images:                 vec![],
			opening_times:          vec![],
			tags:                   vec![],
			approved_at:            location.approved_at,
			approved_by:            None,
			created_at:             location.created_at,
			created_by:             None,
			updated_at:             location.updated_at,
			updated_by:             None,
		}
	}
}

impl From<(Location, Translation, Translation)> for LocationResponse {
	fn from(
		(location, description, excerpt): (Location, Translation, Translation),
	) -> Self {
		Self {
			id:                     location.id,
			name:                   location.name,
			authority:              None,
			description:            Some(description.into()),
			excerpt:                Some(excerpt.into()),
			seat_count:             location.seat_count,
			is_reservable:          location.is_reservable,
			reservation_block_size: location.reservation_block_size,
			min_reservation_length: location.min_reservation_length,
			max_reservation_length: location.max_reservation_length,
			is_visible:             location.is_visible,
			street:                 location.street,
			number:                 location.number,
			zip:                    location.zip,
			city:                   location.city,
			province:               location.province,
			country:                location.country,
			latitude:               location.latitude,
			longitude:              location.longitude,
			images:                 vec![],
			opening_times:          vec![],
			tags:                   vec![],
			approved_at:            location.approved_at,
			approved_by:            None,
			created_at:             location.created_at,
			created_by:             None,
			updated_at:             location.updated_at,
			updated_by:             None,
		}
	}
}

impl
	From<(
		Location,
		Translation,
		Translation,
		Vec<OpeningTime>,
		Option<Profile>,
		Option<Profile>,
		Option<Profile>,
	)> for LocationResponse
{
	fn from(
		(
			location,
			description,
			excerpt,
			opening_times,
			approver,
			creater,
			updater,
		): (
			Location,
			Translation,
			Translation,
			Vec<OpeningTime>,
			Option<Profile>,
			Option<Profile>,
			Option<Profile>,
		),
	) -> Self {
		let opening_times = opening_times.into_iter().map(Into::into).collect();

		Self {
			id: location.id,
			name: location.name,
			authority: None,
			seat_count: location.seat_count,
			is_reservable: location.is_reservable,
			reservation_block_size: location.reservation_block_size,
			min_reservation_length: location.min_reservation_length,
			max_reservation_length: location.max_reservation_length,
			is_visible: location.is_visible,
			street: location.street,
			number: location.number,
			zip: location.zip,
			city: location.city,
			province: location.province,
			country: location.country,
			latitude: location.latitude,
			longitude: location.longitude,
			description: Some(description.into()),
			excerpt: Some(excerpt.into()),
			images: vec![],
			opening_times,
			tags: vec![],
			approved_at: location.approved_at,
			approved_by: approver.map(Into::into),
			created_at: location.created_at,
			created_by: creater.map(Into::into),
			updated_at: location.updated_at,
			updated_by: updater.map(Into::into),
		}
	}
}
