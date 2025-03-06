use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::DbConn;
use crate::error::Error;
use crate::schema::profile;

#[derive(Clone, DbEnum, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::ProfileState"]
pub enum ProfileState {
	PendingEmailVerification,
	Active,
	Disabled,
}

/// A single profile
#[derive(Clone, Debug, Identifiable, Queryable, Selectable, Serialize)]
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
	pub state:                           ProfileState,
	pub created_at:                      NaiveDateTime,
	pub last_login_at:                   NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = profile)]
pub struct InsertableProfile {
	pub username:                        String,
	#[serde(skip)]
	pub password_hash:                   String,
	#[serde(skip)]
	pub pending_email:                   String,
	#[serde(skip)]
	pub email_confirmation_token:        String,
	#[serde(skip)]
	pub email_confirmation_token_expiry: NaiveDateTime,
}

impl InsertableProfile {
	/// Insert this [`InsertableProfile`]
	pub(crate) async fn insert(self, conn: DbConn) -> Result<Profile, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::insert_into(profile)
					.values(self)
					.returning(Profile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(profile)
	}
}

impl Profile {
	/// Get a list of all [`Profile`]s
	pub(crate) async fn get_all(conn: DbConn) -> Result<Vec<Self>, Error> {
		use self::profile::dsl::*;

		let profiles = conn.interact(|conn| profile.load(conn)).await??;

		Ok(profiles)
	}

	/// Get a profile given its email confirmation token
	pub(crate) async fn get_by_email_confirmation_token(
		token: String,
		conn: DbConn,
	) -> Result<Self, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				profile.filter(email_confirmation_token.eq(token)).first(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Confirm the pending email for a [`Profile`]
	pub(crate) async fn confirm_email(
		&self,
		conn: DbConn,
	) -> Result<(), Error> {
		let self_id = self.id;
		let pending = self.pending_email.clone().unwrap();

		conn.interact(move |conn| {
			use self::profile::dsl::*;

			diesel::update(profile.find(self_id))
				.set((
					email.eq(pending),
					pending_email.eq(None::<String>),
					email_confirmation_token.eq(None::<String>),
					email_confirmation_token_expiry.eq(None::<NaiveDateTime>),
				))
				.execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Get a [`Profile`] given its username
	pub(crate) async fn get_by_username(
		query_username: String,
		conn: DbConn,
	) -> Result<Self, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				profile.filter(username.eq(query_username)).first(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Get a [`Profile`] given its email
	pub(crate) async fn get_by_email(
		query_email: String,
		conn: DbConn,
	) -> Result<Self, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				profile.filter(email.eq(query_email)).first(conn)
			})
			.await??;

		Ok(profile)
	}
}
