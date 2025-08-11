use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use lettre::message::Mailbox;
use serde::{Deserialize, Serialize};

use crate::db::{location, opening_time, profile, reservation};
use crate::{
	Image,
	NewImage,
	QUERY_HARD_LIMIT,
	ReservationState,
	manual_pagination,
};

#[derive(
	Clone, Copy, DbEnum, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[ExistingTypePath = "crate::db::sql_types::ProfileState"]
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

impl TryFrom<&PrimitiveProfile> for Mailbox {
	type Error = Error;

	fn try_from(value: &PrimitiveProfile) -> Result<Mailbox, Error> {
		if value.pending_email.is_some() {
			Ok(Mailbox::new(
				Some(value.username.clone()),
				value.pending_email.as_ref().unwrap().parse()?,
			))
		} else if value.email.is_some() {
			Ok(Mailbox::new(
				Some(value.username.clone()),
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

#[derive(Clone, Debug, Queryable, Serialize)]
#[diesel(table_name = profile)]
#[diesel(check_for_backend(Pg))]
pub struct Profile {
	pub profile:    PrimitiveProfile,
	pub avatar_url: Option<String>,
}

impl PrimitiveProfile {
	/// Get a [`Profile`] given its id
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
	#[instrument(skip(conn))]
	pub async fn update(self, conn: &DbConn) -> Result<Self, Error> {
		let new = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::update(profile.find(self.id))
					.set(self)
					.returning(PrimitiveProfile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(new)
	}

	/// Get a list of all [`Profile`]s
	#[instrument(skip(conn))]
	pub async fn get_all(
		limit: usize,
		offset: usize,
		conn: &DbConn,
	) -> Result<(usize, bool, Vec<Self>), Error> {
		use self::profile::dsl::*;

		let profiles = conn
			.interact(move |conn| {
				profile.order_by(id).limit(QUERY_HARD_LIMIT).get_results(conn)
			})
			.await??;

		manual_pagination(profiles, limit, offset)
	}

	/// Check if a [`Profile`] with a given id exists
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

	/// Get a [`Profile`] given a email or username.
	#[instrument(skip(conn))]
	pub async fn get_by_email_or_username(
		query: String,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;
				profile
					.filter(email.eq(&query).or(username.eq(&query)))
					.first(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Get a profile given its email confirmation token
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
	#[instrument(skip(conn))]
	pub async fn confirm_email(&self, conn: &DbConn) -> Result<Self, Error> {
		let self_id = self.id;
		let pending = self.pending_email.clone().unwrap();

		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				diesel::update(profile.find(self_id))
					.set((
						email.eq(pending),
						pending_email.eq(None::<String>),
						email_confirmation_token.eq(None::<String>),
						email_confirmation_token_expiry
							.eq(None::<NaiveDateTime>),
						state.eq(ProfileState::Active),
					))
					.returning(Self::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Set a new email confirmation token and expiry for a [`Profile`]
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
	pub fn hash_password(password: &str) -> Result<String, Error> {
		let salt = SaltString::generate(&mut OsRng);
		let hashed_password = Argon2::default()
			.hash_password(password.as_bytes(), &salt)?
			.to_string();

		Ok(hashed_password)
	}

	/// Change the password for a [`Profile`]
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
					.returning(PrimitiveProfile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(profile)
	}

	/// Set the `last_login_at` field to the current datetime for the given
	/// [`Profile`]
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

	/// Insert an [avatar](NewImage) for this [`Profile`]
	#[instrument(skip(conn))]
	pub async fn insert_avatar(
		p_id: i32,
		avatar: NewImage,
		conn: &DbConn,
	) -> Result<Image, Error> {
		let image = conn
			.interact(move |conn| {
				conn.transaction::<Image, Error, _>(|conn| {
					use crate::db::image::dsl::*;
					use crate::db::profile::dsl::*;

					let image_record = diesel::insert_into(image)
						.values(avatar)
						.returning(Image::as_returning())
						.get_result(conn)?;

					diesel::update(profile)
						.set(avatar_image_id.eq(image_record.id))
						.execute(conn)?;

					Ok(image_record)
				})
			})
			.await??;

		Ok(image)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewProfile {
	pub username:                        String,
	pub password:                        String,
	pub pending_email:                   String,
	pub email_confirmation_token:        String,
	pub email_confirmation_token_expiry: NaiveDateTime,
	pub first_name:                      String,
	pub last_name:                       String,
}

#[derive(Clone, Debug, Insertable)]
#[diesel(table_name = profile)]
struct NewProfileHashed {
	username:                        String,
	password_hash:                   String,
	pending_email:                   String,
	email_confirmation_token:        String,
	email_confirmation_token_expiry: NaiveDateTime,
	first_name:                      String,
	last_name:                       String,
}

impl NewProfile {
	/// Insert this [`NewProfile`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		conn: &DbConn,
	) -> Result<PrimitiveProfile, Error> {
		let hash = PrimitiveProfile::hash_password(&self.password)?;

		let insertable = NewProfileHashed {
			username:                        self.username,
			password_hash:                   hash,
			pending_email:                   self.pending_email,
			email_confirmation_token:        self.email_confirmation_token,
			email_confirmation_token_expiry: self
				.email_confirmation_token_expiry,
			first_name:                      self.first_name,
			last_name:                       self.last_name,
		};

		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::insert_into(profile)
					.values(insertable)
					.returning(PrimitiveProfile::as_returning())
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
	pub async fn insert(
		self,
		conn: &DbConn,
	) -> Result<PrimitiveProfile, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::insert_into(profile)
					.values(self)
					.returning(PrimitiveProfile::as_returning())
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
	pub async fn apply_to(
		self,
		target_id: i32,
		conn: &DbConn,
	) -> Result<PrimitiveProfile, Error> {
		let new = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				diesel::update(profile.find(target_id))
					.set(self)
					.returning(PrimitiveProfile::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(new)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfileStats {
	pub total_reservations:      usize,
	pub completed_reservations:  usize,
	pub upcoming_reservations:   usize,
	pub total_reservation_hours: usize,
}

impl ProfileStats {
	/// Get reservation statistics for a profile
	#[instrument(skip(conn))]
	#[allow(clippy::cast_sign_loss)]
	pub async fn for_profile(
		profile_id: i32,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let reservation_data = conn
			.interact(move |c| {
				use location::dsl as l_dsl;
				use opening_time::dsl as ot_dsl;
				use reservation::dsl as r_dsl;

				r_dsl::reservation
					.inner_join(
						ot_dsl::opening_time
							.on(r_dsl::opening_time_id.eq(ot_dsl::id)),
					)
					.inner_join(
						l_dsl::location.on(ot_dsl::location_id.eq(l_dsl::id)),
					)
					.filter(r_dsl::profile_id.eq(profile_id))
					.select((
						r_dsl::block_count,
						l_dsl::reservation_block_size,
						ot_dsl::day,
						ot_dsl::end_time,
						r_dsl::state,
					))
					.load::<(
						i32,
						i32,
						chrono::NaiveDate,
						chrono::NaiveTime,
						ReservationState,
					)>(c)
			})
			.await??;

		let now = Utc::now().naive_utc();
		let mut total_reservations: usize = 0;
		let mut completed_reservations: usize = 0;
		let mut upcoming_reservations: usize = 0;
		let mut total_reservation_hours: usize = 0;

		for data in reservation_data {
			let (block_count, block_size_minutes, day, end_time, state) = data;

			// Calculate total hours for this reservation
			let reservation_minutes = block_count * block_size_minutes;
			total_reservation_hours += (reservation_minutes as usize) / 60;

			// Determine if reservation is past or future
			let reservation_end = day.and_time(end_time);

			if reservation_end > now {
				if state != ReservationState::Cancelled {
					upcoming_reservations += 1;
				}
			} else {
				completed_reservations += 1;
			}

			total_reservations += 1;
		}

		let stats = ProfileStats {
			total_reservations,
			completed_reservations,
			upcoming_reservations,
			total_reservation_hours,
		};

		Ok(stats)
	}
}
