use chrono::NaiveDateTime;
use models::{
	FullLocationData,
	LocationProfileUpdate,
	LocationUpdate,
	NewLocation,
	NewLocationProfile,
	SimpleProfile,
};
use serde::{Deserialize, Serialize};

use crate::schemas::authority::AuthorityResponse;
use crate::schemas::image::ImageResponse;
use crate::schemas::opening_time::OpeningTimeResponse;
use crate::schemas::ser_includes;
use crate::schemas::tag::TagResponse;
use crate::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
};

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationResponse {
	pub id:                     i32,
	pub name:                   String,
	#[serde(serialize_with = "ser_includes")]
	pub authority:              Option<Option<AuthorityResponse>>,
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
	pub approved_at:            Option<NaiveDateTime>,
	#[serde(serialize_with = "ser_includes")]
	pub approved_by:            Option<Option<SimpleProfile>>,
	pub rejected_at:            Option<NaiveDateTime>,
	#[serde(serialize_with = "ser_includes")]
	pub rejected_by:            Option<Option<SimpleProfile>>,
	pub rejected_reason:        Option<String>,
	pub created_at:             NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub created_by:             Option<Option<SimpleProfile>>,
	pub updated_at:             NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub updated_by:             Option<Option<SimpleProfile>>,

	pub images:        Vec<ImageResponse>,
	pub opening_times: Vec<OpeningTimeResponse>,
	pub tags:          Vec<TagResponse>,
}

impl From<FullLocationData> for LocationResponse {
	fn from(
		(location, (opening_times, tags, images)): FullLocationData,
	) -> Self {
		Self {
			id:                     location.location.id,
			name:                   location.location.name,
			authority:              location
				.authority
				.map(|inner| inner.map(Into::into)),
			description:            Some(location.description.into()),
			excerpt:                Some(location.excerpt.into()),
			seat_count:             location.location.seat_count,
			is_reservable:          location.location.is_reservable,
			reservation_block_size: location.location.reservation_block_size,
			min_reservation_length: location.location.min_reservation_length,
			max_reservation_length: location.location.max_reservation_length,
			is_visible:             location.location.is_visible,
			street:                 location.location.street,
			number:                 location.location.number,
			zip:                    location.location.zip,
			city:                   location.location.city,
			province:               location.location.province,
			country:                location.location.country,
			latitude:               location.location.latitude,
			longitude:              location.location.longitude,
			approved_at:            location.location.approved_at,
			approved_by:            location.approved_by,
			rejected_at:            location.location.rejected_at,
			rejected_by:            location.rejected_by,
			rejected_reason:        location.location.rejected_reason,
			created_at:             location.location.created_at,
			created_by:             location.created_by,
			updated_at:             location.location.updated_at,
			updated_by:             location.updated_by,

			opening_times: opening_times.into_iter().map(Into::into).collect(),
			tags:          tags.into_iter().map(Into::into).collect(),
			images:        images.into_iter().map(Into::into).collect(),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRequest {
	pub name:                   String,
	pub description:            CreateTranslationRequest,
	pub excerpt:                CreateTranslationRequest,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub reservation_block_size: i32,
	pub min_reservation_length: Option<i32>,
	pub max_reservation_length: Option<i32>,
	pub street:                 String,
	pub number:                 String,
	pub zip:                    String,
	pub city:                   String,
	pub province:               String,
	pub country:                String,
	pub latitude:               f64,
	pub longitude:              f64,
}

impl CreateLocationRequest {
	#[must_use]
	pub fn to_insertable(self, created_by: i32) -> NewLocation {
		NewLocation {
			name: self.name,
			authority_id: None,
			description: self.description.to_insertable(created_by),
			excerpt: self.excerpt.to_insertable(created_by),
			seat_count: self.seat_count,
			is_reservable: self.is_reservable,
			reservation_block_size: self.reservation_block_size,
			min_reservation_length: self.min_reservation_length,
			max_reservation_length: self.max_reservation_length,
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

	#[must_use]
	pub fn to_insertable_for_authority(
		self,
		auth_id: i32,
		created_by: i32,
	) -> NewLocation {
		NewLocation {
			name: self.name,
			authority_id: Some(auth_id),
			description: self.description.to_insertable(created_by),
			excerpt: self.excerpt.to_insertable(created_by),
			seat_count: self.seat_count,
			is_reservable: self.is_reservable,
			reservation_block_size: self.reservation_block_size,
			min_reservation_length: self.min_reservation_length,
			max_reservation_length: self.max_reservation_length,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationMemberRequest {
	pub profile_id:  i32,
	pub permissions: i64,
}

impl CreateLocationMemberRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		location_id: i32,
		added_by: i32,
	) -> NewLocationProfile {
		NewLocationProfile {
			location_id,
			profile_id: self.profile_id,
			added_by,
			permissions: self.permissions,
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationRequest {
	pub name:          Option<String>,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
	pub is_visible:    Option<bool>,
	pub street:        Option<String>,
	pub number:        Option<String>,
	pub zip:           Option<String>,
	pub city:          Option<String>,
	pub province:      Option<String>,
	pub latitude:      Option<f64>,
	pub longitude:     Option<f64>,
}

impl UpdateLocationRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> LocationUpdate {
		LocationUpdate {
			name: self.name,
			seat_count: self.seat_count,
			is_reservable: self.is_reservable,
			is_visible: self.is_visible,
			street: self.street,
			number: self.number,
			zip: self.zip,
			city: self.city,
			province: self.province,
			latitude: self.latitude,
			longitude: self.longitude,
			updated_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationMemberRequest {
	pub permissions: i64,
}

impl UpdateLocationMemberRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> LocationProfileUpdate {
		LocationProfileUpdate { updated_by, permissions: self.permissions }
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RejectLocationRequest {
	pub reason: Option<String>,
}
