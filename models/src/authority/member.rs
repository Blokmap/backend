use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{authority_profile, simple_profile};
use crate::{Authority, SimpleProfile};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = authority_profile)]
#[diesel(primary_key(authority_id, profile_id))]
#[diesel(check_for_backend(Pg))]
pub struct AuthorityProfile {
	pub authority_id: i32,
	pub profile_id:   i32,
	pub added_at:     NaiveDateTime,
	pub added_by:     Option<i32>,
	pub updated_at:   NaiveDateTime,
	pub updated_by:   Option<i32>,
	pub permissions:  i64,
}

bitflags! {
	#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
	pub struct Permissions: i64 {
		const Administrator = 1 << 0;
		const AddLocation = 1 << 1;
		const ApproveLocation = 1 << 2;
		const DeleteLocation = 1 << 3;
		const ManageOpeningTimes = 1 << 4;
		const ManageReservations = 1 << 5;
		const ManageMembers = 1 << 6;
	}
}

impl Authority {
	/// Get all [members](SimpleProfile) of this [`Authority`]
	#[instrument(skip(conn))]
	pub async fn get_members(
		auth_id: i32,
		conn: &DbConn,
	) -> Result<Vec<SimpleProfile>, Error> {
		let members = conn
			.interact(move |conn| {
				authority_profile::table
					.filter(authority_profile::authority_id.eq(auth_id))
					.inner_join(simple_profile::table.on(
						simple_profile::id.eq(authority_profile::profile_id),
					))
					.select(SimpleProfile::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(members)
	}

	/// Get all [members](SimpleProfile) of this [`Authority`] alongside their
	/// permissions
	#[instrument(skip(conn))]
	pub async fn get_members_with_permissions(
		auth_id: i32,
		conn: &DbConn,
	) -> Result<Vec<(SimpleProfile, Permissions)>, Error> {
		let members = conn
			.interact(move |conn| {
				authority_profile::table
					.filter(authority_profile::authority_id.eq(auth_id))
					.inner_join(simple_profile::table.on(
						simple_profile::id.eq(authority_profile::profile_id),
					))
					.select((
						SimpleProfile::as_select(),
						authority_profile::permissions,
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(prof, perm): (_, i64)| {
				let perm = Permissions::from_bits_truncate(perm);
				(prof, perm)
			})
			.collect();

		Ok(members)
	}

	/// Delete a member from this authority
	#[instrument(skip(conn))]
	pub async fn delete_member(
		auth_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::schema::authority_profile::dsl::*;

			diesel::delete(authority_profile.find((auth_id, prof_id)))
				.execute(conn)
		})
		.await??;

		info!("deleted profile {prof_id} from authority {auth_id}");

		Ok(())
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = authority_profile)]
#[diesel(check_for_backend(Pg))]
pub struct NewAuthorityProfile {
	pub authority_id: i32,
	pub profile_id:   i32,
	pub added_by:     i32,
	pub permissions:  i64,
}

impl NewAuthorityProfile {
	/// Insert this [`NewAuthorityProfile`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		conn: &DbConn,
	) -> Result<(SimpleProfile, Permissions), Error> {
		conn.interact(move |conn| {
			use crate::schema::authority_profile::dsl::*;

			diesel::insert_into(authority_profile).values(self).execute(conn)
		})
		.await??;

		let (profile, permissions): (SimpleProfile, i64) = conn
			.interact(move |conn| {
				authority_profile::table
					.filter(
						authority_profile::authority_id
							.eq(self.authority_id)
							.and(
								authority_profile::profile_id
									.eq(self.profile_id),
							),
					)
					.inner_join(simple_profile::table.on(
						simple_profile::id.eq(authority_profile::profile_id),
					))
					.select((
						SimpleProfile::as_select(),
						authority_profile::permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions = Permissions::from_bits_truncate(permissions);

		info!(
			"added profile {} to authority {}",
			self.profile_id, self.authority_id
		);

		Ok((profile, permissions))
	}
}

#[derive(AsChangeset, Clone, Copy, Debug, Deserialize, Serialize)]
#[diesel(table_name = authority_profile)]
#[diesel(check_for_backend(Pg))]
pub struct AuthorityProfileUpdate {
	pub updated_by:  i32,
	pub permissions: i64,
}

impl AuthorityProfileUpdate {
	/// Apply this update to the [`Authority`] with the given id
	pub async fn apply_to(
		self,
		auth_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(SimpleProfile, Permissions), Error> {
		conn.interact(move |conn| {
			use crate::schema::authority_profile::dsl::*;

			diesel::update(authority_profile.find((auth_id, prof_id)))
				.set(self)
				.execute(conn)
		})
		.await??;

		let (profile, permissions): (SimpleProfile, i64) = conn
			.interact(move |conn| {
				authority_profile::table
					.find((auth_id, prof_id))
					.inner_join(simple_profile::table.on(
						simple_profile::id.eq(authority_profile::profile_id),
					))
					.select((
						SimpleProfile::as_select(),
						authority_profile::permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions = Permissions::from_bits_truncate(permissions);

		info!(
			"set permissions for profile {} to {} in authority {}",
			prof_id, self.permissions, auth_id
		);

		Ok((profile, permissions))
	}
}
