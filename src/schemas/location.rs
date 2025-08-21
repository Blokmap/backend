use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromMultipart};
use chrono::NaiveDateTime;
use common::Error;
use models::{
	FullLocationData,
	LocationProfileUpdate,
	LocationUpdate,
	NewLocation,
	NewLocationProfile,
	PrimitiveLocation,
};
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

use crate::Config;
use crate::schemas::authority::AuthorityResponse;
use crate::schemas::image::ImageResponse;
use crate::schemas::opening_time::OpeningTimeResponse;
use crate::schemas::profile::ProfileResponse;
use crate::schemas::tag::TagResponse;
use crate::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
};
use crate::schemas::{BuildResponse, ser_includes};

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NearestLocationResponse {
	pub id:        i32,
	pub latitude:  f64,
	pub longitude: f64,
}

impl From<(i32, f64, f64)> for NearestLocationResponse {
	fn from(value: (i32, f64, f64)) -> Self {
		Self { id: value.0, latitude: value.1, longitude: value.2 }
	}
}

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
	pub approved_by:            Option<Option<ProfileResponse>>,
	pub rejected_at:            Option<NaiveDateTime>,
	#[serde(serialize_with = "ser_includes")]
	pub rejected_by:            Option<Option<ProfileResponse>>,
	pub rejected_reason:        Option<String>,
	pub created_at:             NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub created_by:             Option<Option<ProfileResponse>>,
	pub updated_at:             NaiveDateTime,
	#[serde(serialize_with = "ser_includes")]
	pub updated_by:             Option<Option<ProfileResponse>>,

	pub images:        Vec<ImageResponse>,
	pub opening_times: Vec<OpeningTimeResponse>,
	pub tags:          Vec<TagResponse>,
}

impl From<PrimitiveLocation> for LocationResponse {
	fn from(value: PrimitiveLocation) -> Self {
		Self {
			id:                     value.id,
			name:                   value.name,
			authority:              None,
			description:            None,
			excerpt:                None,
			seat_count:             value.seat_count,
			is_reservable:          value.is_reservable,
			max_reservation_length: value.max_reservation_length,
			is_visible:             value.is_visible,
			street:                 value.street,
			number:                 value.number,
			zip:                    value.zip,
			city:                   value.city,
			province:               value.province,
			country:                value.country,
			latitude:               value.latitude,
			longitude:              value.longitude,
			approved_at:            value.approved_at,
			approved_by:            None,
			rejected_at:            value.rejected_at,
			rejected_by:            None,
			rejected_reason:        value.rejected_reason,
			created_at:             value.created_at,
			created_by:             None,
			updated_at:             value.updated_at,
			updated_by:             None,

			opening_times: vec![],
			tags:          vec![],
			images:        vec![],
		}
	}
}

impl BuildResponse<LocationResponse> for FullLocationData {
	fn build_response(
		self,
		config: &Config,
	) -> Result<LocationResponse, Error> {
		let (location, (opening_times, tags, images)) = self;

		Ok(LocationResponse {
			id:                     location.location.id,
			name:                   location.location.name,
			authority:              location
				.authority
				.map(|inner| inner.map(Into::into)),
			description:            Some(location.description.into()),
			excerpt:                Some(location.excerpt.into()),
			seat_count:             location.location.seat_count,
			is_reservable:          location.location.is_reservable,
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
			approved_by:            location
				.approved_by
				.map(|p| p.map(Into::into)),
			rejected_at:            location.location.rejected_at,
			rejected_by:            location
				.rejected_by
				.map(|p| p.map(Into::into)),
			rejected_reason:        location.location.rejected_reason,
			created_at:             location.location.created_at,
			created_by:             location
				.created_by
				.map(|p| p.map(Into::into)),
			updated_at:             location.location.updated_at,
			updated_by:             location
				.updated_by
				.map(|p| p.map(Into::into)),

			opening_times: opening_times.into_iter().map(Into::into).collect(),
			tags:          tags.into_iter().map(Into::into).collect(),
			images:        images
				.into_iter()
				.map(|i| i.build_response(config))
				.collect::<Result<_, _>>()?,
		})
	}
}

#[derive(Debug, TryFromMultipart)]
#[try_from_multipart(rename_all = "camelCase")]
pub struct CreateLocationImageRequest {
	pub images:  Vec<FieldData<Bytes>>,
	pub indices: Vec<FieldData<i32>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
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
	#[validate(length(equal = 2))]
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
