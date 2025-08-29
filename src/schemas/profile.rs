use chrono::NaiveDateTime;
use common::Error;
use primitives::PrimitiveProfile;
use profile::{Profile, ProfileStats, UpdateProfile};
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::schemas::BuildResponse;
use crate::schemas::image::ImageResponse;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
	pub id:            i32,
	pub username:      String,
	pub email:         Option<String>,
	pub first_name:    Option<String>,
	pub last_name:     Option<String>,
	pub is_admin:      bool,
	pub created_at:    NaiveDateTime,
	pub last_login_at: NaiveDateTime,
	pub avatar_url:    Option<ImageResponse>,
}

impl From<PrimitiveProfile> for ProfileResponse {
	fn from(value: PrimitiveProfile) -> Self {
		Self {
			id:            value.id,
			username:      value.username,
			email:         value.email,
			first_name:    value.first_name,
			last_name:     value.last_name,
			is_admin:      value.is_admin,
			created_at:    value.created_at,
			last_login_at: value.last_login_at,
			avatar_url:    None,
		}
	}
}

impl BuildResponse<ProfileResponse> for Profile {
	type Includes = ();

	fn build_response(
		self,
		_includes: Self::Includes,
		config: &Config,
	) -> Result<ProfileResponse, Error> {
		Ok(ProfileResponse {
			id:            self.primitive.id,
			username:      self.primitive.username,
			email:         self.primitive.email,
			first_name:    self.primitive.first_name,
			last_name:     self.primitive.last_name,
			is_admin:      self.primitive.is_admin,
			created_at:    self.primitive.created_at,
			last_login_at: self.primitive.last_login_at,
			avatar_url:    self
				.avatar
				.map(|i| i.build_response((), config))
				.transpose()?,
		})
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
	pub username:      Option<String>,
	pub pending_email: Option<String>,
}

impl From<UpdateProfileRequest> for UpdateProfile {
	fn from(request: UpdateProfileRequest) -> Self {
		Self {
			username:      request.username,
			pending_email: request.pending_email,
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatsResponse {
	pub total_reservations:      usize,
	pub completed_reservations:  usize,
	pub upcoming_reservations:   usize,
	pub total_reservation_hours: usize,
}

impl From<ProfileStats> for ProfileStatsResponse {
	fn from(stats: ProfileStats) -> Self {
		Self {
			total_reservations:      stats.total_reservations,
			completed_reservations:  stats.completed_reservations,
			upcoming_reservations:   stats.upcoming_reservations,
			total_reservation_hours: stats.total_reservation_hours,
		}
	}
}
