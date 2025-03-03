use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::Serialize;

use crate::DbConn;
use crate::error::Error;
use crate::schema::profile;

#[derive(Clone, DbEnum, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::UserState"]
pub enum UserState {
	PendingEmailVerification,
	Active,
	Disabled,
}

/// A single profile
#[derive(Clone, Debug, Identifiable, Queryable, Serialize)]
#[diesel(table_name = profile)]
pub struct Profile {
	pub id:                              i32,
	pub username:                        String,
	#[serde(skip)]
	pub password_hash:                   String,
	#[serde(skip)]
	pub password_reset_token:            Option<String>,
	#[serde(skip)]
	pub password_reset_token_expiry:     Option<NaiveDateTime>,
	pub email:                           Option<String>,
	#[serde(skip)]
	pub pending_email:                   Option<String>,
	#[serde(skip)]
	pub email_confirmation_token:        Option<String>,
	#[serde(skip)]
	pub email_confirmation_token_expiry: Option<NaiveDateTime>,
	pub admin:                           bool,
	#[serde(skip)]
	pub state:                           UserState,
	pub created_at:                      NaiveDateTime,
}

impl Profile {
	/// Get a list of all profiles
	pub(crate) async fn get_all(conn: DbConn) -> Result<Vec<Self>, Error> {
		use self::profile::dsl::*;

		let profiles = conn.interact(|conn| profile.load(conn)).await??;

		Ok(profiles)
	}
}
