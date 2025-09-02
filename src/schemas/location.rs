use chrono::NaiveDateTime;
use common::Error;
use image::{ImageIncludes, NewLocationImage};
use location::{
	FullLocationData,
	LocationIncludes,
	LocationMemberUpdate,
	LocationUpdate,
	NewLocation,
	NewLocationMember,
};
use opening_time::OpeningTimeIncludes;
use primitives::PrimitiveLocation;
use serde::{Deserialize, Serialize};
use tag::TagIncludes;
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
	type Includes = LocationIncludes;

	fn build_response(
		self,
		includes: Self::Includes,
		config: &Config,
	) -> Result<LocationResponse, Error> {
		let (location, (opening_times, tags, images)) = self;

		let authority = location.authority.map(Into::into);
		let approved_by = location.approved_by.map(Into::into);
		let rejected_by = location.rejected_by.map(Into::into);
		let created_by = location.created_by.map(Into::into);
		let updated_by = location.updated_by.map(Into::into);

		Ok(LocationResponse {
			id:                     location.primitive.id,
			name:                   location.primitive.name,
			authority:              if includes.authority {
				Some(authority)
			} else {
				None
			},
			description:            Some(location.description.into()),
			excerpt:                Some(location.excerpt.into()),
			seat_count:             location.primitive.seat_count,
			is_reservable:          location.primitive.is_reservable,
			max_reservation_length: location.primitive.max_reservation_length,
			is_visible:             location.primitive.is_visible,
			street:                 location.primitive.street,
			number:                 location.primitive.number,
			zip:                    location.primitive.zip,
			city:                   location.primitive.city,
			province:               location.primitive.province,
			country:                location.primitive.country,
			latitude:               location.primitive.latitude,
			longitude:              location.primitive.longitude,
			approved_at:            location.primitive.approved_at,
			approved_by:            if includes.approved_by {
				Some(approved_by)
			} else {
				None
			},
			rejected_at:            location.primitive.rejected_at,
			rejected_by:            if includes.rejected_by {
				Some(rejected_by)
			} else {
				None
			},
			rejected_reason:        location.primitive.rejected_reason,
			created_at:             location.primitive.created_at,
			created_by:             if includes.created_by {
				Some(created_by)
			} else {
				None
			},
			updated_at:             location.primitive.updated_at,
			updated_by:             if includes.updated_by {
				Some(updated_by)
			} else {
				None
			},

			opening_times: opening_times
				.into_iter()
				.map(|t| {
					t.build_response(OpeningTimeIncludes::default(), config)
				})
				.collect::<Result<_, _>>()?,
			tags:          tags
				.into_iter()
				.map(|t| t.build_response(TagIncludes::default(), config))
				.collect::<Result<_, _>>()?,
			images:        images
				.into_iter()
				.map(|i| i.build_response(ImageIncludes::default(), config))
				.collect::<Result<_, _>>()?,
		})
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationImageOrderUpdate {
	pub image_id: i32,
	pub index:    i32,
}

impl LocationImageOrderUpdate {
	#[must_use]
	pub fn to_insertable(self, location_id: i32) -> NewLocationImage {
		NewLocationImage {
			location_id,
			image_id: self.image_id,
			index: self.index,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRequest {
	pub name:                   String,
	pub description:            CreateTranslationRequest,
	pub excerpt:                CreateTranslationRequest,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub is_visible:             bool,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationMemberRequest {
	pub profile_id:       i32,
	pub location_role_id: Option<i32>,
}

impl CreateLocationMemberRequest {
	#[must_use]
	pub fn to_insertable(
		self,
		location_id: i32,
		added_by: i32,
	) -> NewLocationMember {
		NewLocationMember {
			location_id,
			profile_id: self.profile_id,
			location_role_id: self.location_role_id,
			added_by,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationMemberUpdateRequest {
	pub location_role_id: Option<i32>,
}

impl LocationMemberUpdateRequest {
	#[must_use]
	pub fn to_insertable(self, updated_by: i32) -> LocationMemberUpdate {
		LocationMemberUpdate {
			location_role_id: self.location_role_id,
			updated_by,
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
pub struct RejectLocationRequest {
	pub reason: Option<String>,
}
