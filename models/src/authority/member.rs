use std::collections::HashMap;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};

use crate::schema::{
	authority,
	authority_profile,
	creator,
	image,
	profile,
	updater,
};
use crate::{
	Authority,
	AuthorityIncludes,
	Image,
	PrimitiveAuthority,
	PrimitiveProfile,
};

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
	/// Possible permissions for a member of an [`Authority`]
	#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
	pub struct AuthorityPermissions: i64 {
		/// Admin privileges, member can do everything
		const Administrator = 1 << 0;
		/// Member can manage the authority itself
		const ManageAuthority = 1 << 1;
		/// Member can manage locations
		const ManageLocation = 1 << 2;
		/// Member can submit new locations
		const AddLocation = 1 << 3;
		/// Member can approve/reject locations
		const ApproveLocation = 1 << 4;
		/// Member can delete locations
		const DeleteLocation = 1 << 5;
		/// Member can manage opening times for locations
		const ManageOpeningTimes = 1 << 6;
		/// Member can manage reservations on the authorities locations
		const ManageReservations = 1 << 7;
		/// Member can manage authority members
		const ManageMembers = 1 << 8;
	}
}

impl AuthorityPermissions {
	#[must_use]
	pub fn names() -> HashMap<&'static str, i64> {
		Self::all().iter_names().map(|(n, v)| (n, v.bits())).collect()
	}
}

impl Authority {
	/// Get all [members](SimpleProfile) of this [`Authority`]
	#[instrument(skip(conn))]
	pub async fn get_members(
		auth_id: i32,
		conn: &DbConn,
	) -> Result<Vec<PrimitiveProfile>, Error> {
		let members = conn
			.interact(move |conn| {
				authority_profile::table
					.filter(authority_profile::authority_id.eq(auth_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(authority_profile::profile_id)),
					)
					.select(PrimitiveProfile::as_select())
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
	) -> Result<
		Vec<(PrimitiveProfile, Option<Image>, AuthorityPermissions)>,
		Error,
	> {
		let members = conn
			.interact(move |conn| {
				authority_profile::table
					.filter(authority_profile::authority_id.eq(auth_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(authority_profile::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						authority_profile::permissions,
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(prof, img, perm): (_, _, i64)| {
				let perm = AuthorityPermissions::from_bits_truncate(perm);
				(prof, img, perm)
			})
			.collect();

		Ok(members)
	}

	/// Get the permissions for a single member
	#[instrument(skip(conn))]
	pub async fn get_member_permissions(
		auth_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<AuthorityPermissions, Error> {
		let permissions = conn
			.interact(move |conn| {
				use crate::schema::authority_profile::dsl::*;

				authority_profile
					.find((auth_id, prof_id))
					.select(permissions)
					.get_result(conn)
			})
			.await??;

		Ok(AuthorityPermissions::from_bits_truncate(permissions))
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

	/// Get all [`Authorities`](Authority) for a given profile
	#[instrument(skip(conn))]
	pub async fn for_profile(
		p_id: i32,
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let authorities = conn
			.interact(move |conn| {
				use crate::schema::authority_profile::dsl::*;

				authority_profile
					.filter(profile_id.eq(p_id))
					.inner_join(
						authority::table.on(authority_id.eq(authority::id)),
					)
					.left_outer_join(
						creator.on(includes.created_by.into_sql::<Bool>().and(
							authority::created_by
								.eq(creator.field(profile::id).nullable()),
						)),
					)
					.left_outer_join(
						updater.on(includes.updated_by.into_sql::<Bool>().and(
							authority::updated_by
								.eq(updater.field(profile::id).nullable()),
						)),
					)
					.select((
						PrimitiveAuthority::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(authority, cr, up)| {
				Authority {
					authority,
					created_by: if includes.created_by {
						Some(cr)
					} else {
						None
					},
					updated_by: if includes.updated_by {
						Some(up)
					} else {
						None
					},
				}
			})
			.collect();

		Ok(authorities)
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
	) -> Result<(PrimitiveProfile, Option<Image>, AuthorityPermissions), Error>
	{
		conn.interact(move |conn| {
			use crate::schema::authority_profile::dsl::*;

			diesel::insert_into(authority_profile).values(self).execute(conn)
		})
		.await??;

		let (profile, img, permissions): (_, _, i64) = conn
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
					.inner_join(
						profile::table
							.on(profile::id.eq(authority_profile::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						authority_profile::permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions = AuthorityPermissions::from_bits_truncate(permissions);

		info!(
			"added profile {} to authority {}",
			self.profile_id, self.authority_id
		);

		Ok((profile, img, permissions))
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
	) -> Result<(PrimitiveProfile, Option<Image>, AuthorityPermissions), Error>
	{
		conn.interact(move |conn| {
			use crate::schema::authority_profile::dsl::*;

			diesel::update(authority_profile.find((auth_id, prof_id)))
				.set(self)
				.execute(conn)
		})
		.await??;

		let (profile, img, permissions): (_, _, i64) = conn
			.interact(move |conn| {
				authority_profile::table
					.find((auth_id, prof_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(authority_profile::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						authority_profile::permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions = AuthorityPermissions::from_bits_truncate(permissions);

		info!(
			"set permissions for profile {} to {} in authority {}",
			prof_id, self.permissions, auth_id
		);

		Ok((profile, img, permissions))
	}
}
