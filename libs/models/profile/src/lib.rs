#[macro_use]
extern crate tracing;

use ::image::NewImage;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use base::{
	PaginatedData,
	PaginationConfig,
	QUERY_HARD_LIMIT,
	RESERVATION_BLOCK_SIZE_MINUTES,
	manual_pagination,
};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use common::{DbConn, Error, OAuthError};
use db::{
	ProfileState,
	ReservationState,
	image,
	opening_time,
	profile,
	reservation,
};
use diesel::prelude::*;
use lettre::message::Mailbox;
use openidconnect::core::CoreGenderClaim;
use openidconnect::{EmptyAdditionalClaims, IdTokenClaims};
use primitives::{PrimitiveImage, PrimitiveProfile};
use rand::Rng;
use rand::distr::Alphabetic;
use serde::{Deserialize, Serialize};

pub type JoinedProfileData = (PrimitiveProfile, Option<PrimitiveImage>);

impl TryFrom<&Profile> for Mailbox {
	type Error = Error;

	fn try_from(value: &Profile) -> Result<Mailbox, Error> {
		let profile = &value.profile;

		if profile.pending_email.is_some() {
			Ok(Mailbox::new(
				Some(profile.username.clone()),
				profile.pending_email.as_ref().unwrap().parse()?,
			))
		} else if profile.email.is_some() {
			Ok(Mailbox::new(
				Some(profile.username.clone()),
				profile.email.as_ref().unwrap().parse()?,
			))
		} else {
			error!(
				"mailer error -- failed to create mailbox, no email found for \
				 profile {}",
				profile.id
			);
			Err(Error::InternalServerError)
		}
	}
}

#[derive(Clone, Debug, Queryable, Serialize)]
#[diesel(table_name = profile)]
#[diesel(check_for_backend(Pg))]
pub struct Profile {
	pub profile: PrimitiveProfile,
	pub avatar:  Option<PrimitiveImage>,
}

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl Profile {
	/// Build a query with all required (dynamic) joins to select a full
	/// profile data tuple
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn joined_query() -> _ {
		profile::table.left_outer_join(
			image::table.on(profile::avatar_image_id.eq(image::id.nullable())),
		)
	}

	/// Construct a full [`Profile`] struct from the data returned by a
	/// joined query
	fn from_joined(data: JoinedProfileData) -> Self {
		Self { profile: data.0, avatar: data.1 }
	}

	/// Get a [`Profile`] given its id
	#[instrument(skip(conn))]
	pub async fn get(query_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let query = Self::joined_query();

		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				query
					.filter(id.eq(query_id))
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let profile = Self::from_joined(profile);

		Ok(profile)
	}

	/// Update a given [`Profile`]
	#[instrument(skip(conn))]
	pub async fn update(self, conn: &DbConn) -> Result<Self, Error> {
		let self_id = self.profile.id;

		conn.interact(|conn| {
			use self::profile::dsl::*;

			diesel::update(profile.find(self.profile.id))
				.set(self.profile)
				.execute(conn)
		})
		.await??;

		let profile = Self::get(self_id, conn).await?;

		Ok(profile)
	}

	/// Get a list of all [`Profile`]s
	#[instrument(skip(conn))]
	pub async fn get_all(
		p_cfg: PaginationConfig,
		conn: &DbConn,
	) -> Result<PaginatedData<Vec<Self>>, Error> {
		let query = Self::joined_query();

		let profiles = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				query
					.order_by(id)
					.limit(QUERY_HARD_LIMIT)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(Self::from_joined)
			.collect();

		manual_pagination(profiles, p_cfg)
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
		let query = Self::joined_query();

		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				query
					.filter(username.eq(query_username))
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
					))
					.first(conn)
			})
			.await??;

		let profile = Self::from_joined(profile);

		Ok(profile)
	}

	/// Get a [`Profile`] given a email or username.
	#[instrument(skip(conn))]
	pub async fn get_by_email_or_username(
		search: String,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query();

		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				query
					.filter(email.eq(&search).or(username.eq(&search)))
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
					))
					.first(conn)
			})
			.await??;

		let profile = Self::from_joined(profile);

		Ok(profile)
	}

	/// Get a profile given its email confirmation token
	#[instrument(skip(token, conn))]
	pub async fn get_by_email_confirmation_token(
		token: String,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query();

		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				query
					.filter(email_confirmation_token.eq(token))
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
					))
					.first(conn)
			})
			.await??;

		let profile = Self::from_joined(profile);

		Ok(profile)
	}

	/// Get a profile given its password reset token
	#[instrument(skip(token, conn))]
	pub async fn get_by_password_reset_token(
		token: String,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query();

		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				query
					.filter(password_reset_token.eq(token))
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
					))
					.first(conn)
			})
			.await??;

		let profile = Self::from_joined(profile);

		Ok(profile)
	}

	/// Confirm the pending email for a [`Profile`]
	///
	/// # Panics
	/// Panics if called on a [`Profile`] with no pending email
	#[instrument(skip(conn))]
	pub async fn confirm_email(&self, conn: &DbConn) -> Result<Self, Error> {
		let self_id = self.profile.id;
		let pending = self.profile.pending_email.clone().unwrap();

		conn.interact(move |conn| {
			use self::profile::dsl::*;

			diesel::update(profile.find(self_id))
				.set((
					email.eq(pending),
					pending_email.eq(None::<String>),
					email_confirmation_token.eq(None::<String>),
					email_confirmation_token_expiry.eq(None::<NaiveDateTime>),
					state.eq(ProfileState::Active),
				))
				.execute(conn)
		})
		.await??;

		let profile = Self::get(self_id, conn).await?;

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

		self.profile.email_confirmation_token = Some(token.to_string());
		self.profile.email_confirmation_token_expiry =
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

		self.profile.password_reset_token = Some(token.to_string());
		self.profile.password_reset_token_expiry =
			Some(password_reset_token_expiry);

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
		let self_id = self.profile.id;
		let new_password_hash = Self::hash_password(new_password)?;

		conn.interact(move |conn| {
			use self::profile::dsl::*;

			diesel::update(profile.find(self_id))
				.set((
					password_hash.eq(new_password_hash),
					password_reset_token.eq(None::<String>),
					password_reset_token_expiry.eq(None::<NaiveDateTime>),
				))
				.execute(conn)
		})
		.await??;

		let profile = Self::get(self_id, conn).await?;

		Ok(profile)
	}

	/// Set the `last_login_at` field to the current datetime for the given
	/// [`Profile`]
	#[instrument(skip(conn))]
	pub async fn update_last_login(
		mut self,
		conn: &DbConn,
	) -> Result<Self, Error> {
		self.profile.last_login_at = Utc::now().naive_utc();
		self.update(conn).await
	}

	/// Get or create a [`Profile`] from an external SSO provided email
	///
	/// # Panics
	/// Panics if the user has a *very* weird email
	#[instrument(skip(conn))]
	pub async fn from_sso(
		claims: IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let Some(user_email) = claims.email().map(|e| e.to_string()) else {
			return Err(OAuthError::MissingEmailField.into());
		};

		let username = if let Some(n) = claims.preferred_username()
			&& !(**n).is_empty()
		{
			n.to_string()
		} else {
			let prefix = user_email.split('@').next().unwrap().to_string();

			let mut rng = rand::rng();
			let suffix: String =
				(0..5).map(|_| rng.sample(Alphabetic) as char).collect();

			format!("{prefix}.{suffix}")
		};

		let user_email_ = user_email.clone();

		let query = Self::joined_query();

		let profile: Option<Self> = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				query
					.filter(email.eq(user_email_))
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
					))
					.first(conn)
					.optional()
			})
			.await??
			.map(Self::from_joined);

		if let Some(profile) = profile {
			return Ok(profile);
		}

		let new_profile = NewProfileDirect {
			username,
			email: Some(user_email),
			password_hash: String::new(),
			state: ProfileState::Active,
		};

		let profile = new_profile.insert(conn).await?;

		if let Some(avatar_url) = claims.picture()
			&& let Some(avatar_url) = avatar_url.get(None)
		{
			let avatar = NewImage {
				file_path:   None,
				uploaded_by: profile.profile.id,
				image_url:   Some(avatar_url.to_string()),
			};

			avatar.insert_for_profile(profile.profile.id, conn).await?;
		}

		Ok(profile)
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
	pub async fn insert(self, conn: &DbConn) -> Result<Profile, Error> {
		let hash = Profile::hash_password(&self.password)?;

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

		let profile = Profile::get(profile.id, conn).await?;

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
	pub async fn insert(self, conn: &DbConn) -> Result<Profile, Error> {
		let profile = conn
			.interact(|conn| {
				use self::profile::dsl::*;

				diesel::insert_into(profile)
					.values(self)
					.returning(PrimitiveProfile::as_select())
					.get_result(conn)
			})
			.await??;

		let profile = Profile::get(profile.id, conn).await?;

		info!("direct-inserted new profile with id {}", profile.profile.id);

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
	) -> Result<Profile, Error> {
		let profile = conn
			.interact(move |conn| {
				use self::profile::dsl::*;

				diesel::update(profile.find(target_id))
					.set(self)
					.returning(PrimitiveProfile::as_returning())
					.get_result(conn)
			})
			.await??;

		let profile = Profile::get(profile.id, conn).await?;

		Ok(profile)
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
				use self::opening_time::dsl as ot_dsl;
				use self::reservation::dsl as r_dsl;

				r_dsl::reservation
					.inner_join(
						ot_dsl::opening_time
							.on(r_dsl::opening_time_id.eq(ot_dsl::id)),
					)
					.filter(r_dsl::profile_id.eq(profile_id))
					.select((
						r_dsl::block_count,
						ot_dsl::day,
						ot_dsl::end_time,
						r_dsl::state,
					))
					.load::<(
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
			let (block_count, day, end_time, state) = data;

			// Calculate total hours for this reservation
			let reservation_minutes =
				block_count * RESERVATION_BLOCK_SIZE_MINUTES;
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
