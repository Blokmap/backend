use std::ops::Deref;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};

use crate::schema::profile;
use crate::{DbConn, Error};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProfileId(pub(crate) i32);

impl Deref for ProfileId {
	type Target = i32;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl AsRef<i32> for ProfileId {
	fn as_ref(&self) -> &i32 { &self.0 }
}

impl std::fmt::Display for ProfileId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Clone, DbEnum, Debug, Default, Deserialize, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::ProfileState"]
pub enum ProfileState {
	#[default]
	PendingEmailVerification,
	Active,
	Disabled,
}

/// A single profile
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
pub struct Profile {
	pub id:                              i32,
	pub username:                        String,
	pub first_name:                      Option<String>,
	pub last_name:                       Option<String>,
	pub avatar_image_id:                 Option<i32>,
	pub institution_name:                Option<String>,
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

impl TryFrom<&Profile> for Mailbox {
	type Error = Error;

	fn try_from(value: &Profile) -> Result<Mailbox, Error> {
		if value.pending_email.is_some() {
			Ok(Mailbox::new(
				Some(value.username.to_string()),
				value.pending_email.as_ref().unwrap().parse()?,
			))
		} else if value.email.is_some() {
			Ok(Mailbox::new(
				Some(value.username.to_string()),
				value.email.as_ref().unwrap().parse()?,
			))
		} else {
			error!(
				"mailer error -- failed to create mailbox, no email found for \
				 profile {}",
				value.id
			);
			Err(Error::InternalServerError)
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewProfile {
	pub username:                        String,
	#[serde(skip)]
	pub password:                        String,
	#[serde(skip)]
	pub pending_email:                   String,
	#[serde(skip)]
	pub email_confirmation_token:        String,
	#[serde(skip)]
	pub email_confirmation_token_expiry: NaiveDateTime,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = profile)]
struct NewProfileHashed {
	username:                        String,
	password_hash:                   String,
	pending_email:                   String,
	email_confirmation_token:        String,
	email_confirmation_token_expiry: NaiveDateTime,
}

impl NewProfile {
	/// Insert this [`NewProfile`]
	#[instrument(skip(conn))]
	pub(crate) async fn insert(self, conn: &DbConn) -> Result<Profile, Error> {
		let hash = Profile::hash_password(&self.password)?;

		let insertable = NewProfileHashed {
			username:                        self.username,
			password_hash:                   hash,
			pending_email:                   self.pending_email,
			email_confirmation_token:        self.email_confirmation_token,
			email_confirmation_token_expiry: self
				.email_confirmation_token_expiry,
		};

		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::insert_into(profile)
					.values(insertable)
					.returning(Profile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(profile)
	}
}

/// A new insertable profile that bypasses email verification and has an
/// explicit email
///
/// Used for SSO logins like OAuth/SAML
#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = profile)]
pub struct NewProfileDirect {
	pub username:      String,
	pub password_hash: String,
	pub email:         Option<String>,
	pub state:         ProfileState,
}

impl NewProfileDirect {
	/// Insert this [`NewProfileDirect`]
	#[instrument(skip(conn))]
	pub(crate) async fn insert(self, conn: &DbConn) -> Result<Profile, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::insert_into(profile)
					.values(self)
					.returning(Profile::as_returning())
					.get_result(conn)
			})
			.await??;

		info!("direct-inserted new profile with id {}", profile.id);

		Ok(profile)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize, Serialize)]
#[diesel(table_name = profile)]
pub struct UpdateProfile {
	pub username:      Option<String>,
	pub pending_email: Option<String>,
}

impl UpdateProfile {
	/// Update a [`Profile`] with the given changes
	#[instrument(skip(conn))]
	pub(crate) async fn apply_to(
		self,
		target_id: i32,
		conn: &DbConn,
	) -> Result<Profile, Error> {
		let new = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				diesel::update(profile.find(target_id))
					.set(self)
					.returning(Profile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(new)
	}
}

impl Profile {
	/// Get a [`Profile`] given its id
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn get(query_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let profiles = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				profile.find(query_id).get_result(conn)
			})
			.await??;

		Ok(profiles)
	}

	/// Update a given [`Profile`]
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn update(self, conn: &DbConn) -> Result<Self, Error> {
		let new = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::update(profile.find(self.id))
					.set(self)
					.returning(Profile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(new)
	}

	/// Get a list of all [`Profile`]s
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn get_all(conn: &DbConn) -> Result<Vec<Self>, Error> {
		use self::profile::dsl::*;

		let profiles = conn.interact(|conn| profile.load(conn)).await??;

		Ok(profiles)
	}

	/// Check if a [`Profile`] with a given id exists
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn exists(query_id: i32, conn: &DbConn) -> Result<bool, Error> {
		let exists = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				diesel::select(diesel::dsl::exists(profile.find(query_id)))
					.get_result(conn)
			})
			.await??;

		Ok(exists)
	}

	/// Get a [`Profile`] given its username
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn get_by_username(
		query_username: String,
		conn: &DbConn,
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
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn get_by_email(
		query_email: String,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				profile.filter(email.eq(query_email)).first(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Get a profile given its email confirmation token
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(token, conn))]
	pub async fn get_by_email_confirmation_token(
		token: String,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				profile.filter(email_confirmation_token.eq(token)).first(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Get a profile given its password reset token
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(token, conn))]
	pub async fn get_by_password_reset_token(
		token: String,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				profile.filter(password_reset_token.eq(token)).first(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Confirm the pending email for a [`Profile`]
	///
	/// # Panics
	/// Panics if called on a [`Profile`] with no pending email
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn confirm_email(&self, conn: &DbConn) -> Result<(), Error> {
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

	/// Set a new email confirmation token and expiry for a [`Profile`]
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(token, conn))]
	pub async fn set_email_confirmation_token(
		mut self,
		token: &str,
		lifetime: TimeDelta,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let email_confirmation_token_expiry = Utc::now().naive_utc() + lifetime;

		self.email_confirmation_token = Some(token.to_string());
		self.email_confirmation_token_expiry =
			Some(email_confirmation_token_expiry);

		self.update(conn).await
	}

	/// Set a new password reset token and expiry for a [`Profile`]
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(token, conn))]
	pub async fn set_password_reset_token(
		mut self,
		token: &str,
		lifetime: TimeDelta,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let password_reset_token_expiry = Utc::now().naive_utc() + lifetime;

		self.password_reset_token = Some(token.to_string());
		self.password_reset_token_expiry = Some(password_reset_token_expiry);

		self.update(conn).await
	}

	/// Hash a password using Argon2
	///
	/// # Errors
	/// Errors if hashing the password fails
	pub fn hash_password(password: &str) -> Result<String, Error> {
		let salt = SaltString::generate(&mut OsRng);
		let hashed_password = Argon2::default()
			.hash_password(password.as_bytes(), &salt)?
			.to_string();

		Ok(hashed_password)
	}

	/// Change the password for a [`Profile`]
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(new_password, conn))]
	pub async fn change_password(
		&self,
		new_password: &str,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let self_id = self.id;
		let new_password_hash = Self::hash_password(new_password)?;

		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				diesel::update(profile.find(self_id))
					.set((
						password_hash.eq(new_password_hash),
						password_reset_token.eq(None::<String>),
						password_reset_token_expiry.eq(None::<NaiveDateTime>),
					))
					.returning(Profile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Set the `last_login_at` field to the current datetime for the given
	/// [`Profile`]
	///
	/// # Errors
	/// Errors if interacting with the database fails
	#[instrument(skip(conn))]
	pub async fn update_last_login(
		mut self,
		conn: &DbConn,
	) -> Result<Self, Error> {
		self.last_login_at = Utc::now().naive_utc();
		self.update(conn).await
	}

	/// Get or create a [`Profile`] from an external SSO provided email
	#[instrument(skip(conn))]
	pub async fn from_sso(
		query_email: String,
		username: Option<String>,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query_email_ = query_email.clone();

		let profile: Option<Self> = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				profile.filter(email.eq(query_email_)).first(conn).optional()
			})
			.await??;

		if let Some(profile) = profile {
			return Ok(profile);
		}

		let new_profile = NewProfileDirect {
			username:      username.unwrap_or_default(),
			email:         Some(query_email),
			password_hash: String::new(),
			state:         ProfileState::Active,
		};

		new_profile.insert(conn).await
	}
}
