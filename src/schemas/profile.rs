use bitflags::Flags;
use chrono::NaiveDateTime;
use common::Error;
use models::{
	Image,
	PrimitiveProfile,
	ProfileState,
	ProfileStats,
	UpdateProfile,
};
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
}

impl From<PrimitiveProfile> for ProfileResponse {
	fn from(profile: PrimitiveProfile) -> Self {
		Self {
			id:            profile.id,
			username:      profile.username,
			email:         profile.email,
			first_name:    profile.first_name,
			last_name:     profile.last_name,
			is_admin:      profile.is_admin,
			created_at:    profile.created_at,
			last_login_at: profile.last_login_at,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePermissionsResponse {
	pub id:          i32,
	pub username:    String,
	pub avatar:      Option<ImageResponse>,
	pub email:       Option<String>,
	pub first_name:  Option<String>,
	pub last_name:   Option<String>,
	pub state:       ProfileState,
	pub permissions: i64,
}

impl<F> BuildResponse<ProfilePermissionsResponse>
	for (PrimitiveProfile, Option<Image>, F)
where
	F: Flags<Bits = i64>,
{
	fn build_response(
		self,
		config: &Config,
	) -> Result<ProfilePermissionsResponse, Error> {
		Ok(ProfilePermissionsResponse {
			id:          self.0.id,
			username:    self.0.username,
			avatar:      self
				.1
				.map(|i| i.build_response(config))
				.transpose()?,
			email:       self.0.email,
			first_name:  self.0.first_name,
			last_name:   self.0.last_name,
			state:       self.0.state,
			permissions: self.2.bits(),
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
