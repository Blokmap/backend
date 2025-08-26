use chrono::NaiveDateTime;
use db::{ProfileState, profile};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	AsChangeset,
	Clone,
	Debug,
	Deserialize,
	Identifiable,
	Insertable,
	Queryable,
	Selectable,
	Serialize,
)]
#[diesel(table_name = profile)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveProfile {
	pub id:                              i32,
	pub username:                        String,
	pub first_name:                      Option<String>,
	pub last_name:                       Option<String>,
	pub avatar_image_id:                 Option<i32>,
	pub institution_id:                  Option<i32>,
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
	pub is_admin:                        bool,
	pub block_reason:                    Option<String>,
	#[serde(skip)]
	pub state:                           ProfileState,
	pub created_at:                      NaiveDateTime,
	pub updated_at:                      NaiveDateTime,
	pub updated_by:                      Option<i32>,
	pub last_login_at:                   NaiveDateTime,
}
