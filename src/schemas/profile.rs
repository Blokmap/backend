use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::{Profile, UpdateProfile};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
	pub id:            i32,
	pub username:      String,
	pub email:         Option<String>,
	pub is_admin:      bool,
	pub created_at:    NaiveDateTime,
	pub last_login_at: NaiveDateTime,
}

impl From<Profile> for ProfileResponse {
	fn from(profile: Profile) -> Self {
		Self {
			id:            profile.id,
			username:      profile.username,
			email:         profile.email,
			is_admin:      profile.is_admin,
			created_at:    profile.created_at,
			last_login_at: profile.last_login_at,
		}
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
