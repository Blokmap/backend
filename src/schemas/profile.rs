use chrono::NaiveDateTime;
use models::{Profile, UpdateProfile};
use serde::{Deserialize, Serialize};

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

impl From<Profile> for ProfileResponse {
	fn from(profile: Profile) -> Self {
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
