use std::collections::HashMap;
use std::hash::Hash;

use common::{DbConn, Error};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{image, location_profile, profile};
use crate::{
	AuthorityPermissions,
	Image,
	Location,
	LocationIncludes,
	PrimitiveProfile,
};

bitflags! {
	/// Possible permissions for a member of a [`Location`]
	#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
	pub struct LocationPermissions: i64 {
		/// Admin privileges, member can do everything
		const Administrator = 1 << 0;
		/// Member can manage this location
		const ManageLocation = 1 << 1;
		/// Member can delete this location
		const DeleteLocation = 1 << 2;
		/// Member can manage opening times for this location
		const ManageOpeningTimes = 1 << 3;
		/// Member can manage reservations for this location
		const ManageReservations = 1 << 4;
		/// Member can manage this locations members
		const ManageMembers = 1 << 5;
	}
}

impl LocationPermissions {
	#[must_use]
	pub fn names() -> HashMap<&'static str, i64> {
		Self::all().iter_names().map(|(n, v)| (n, v.bits())).collect()
	}
}

impl Location {
	/// Check if the given profile is an admin/owner of the given location or
	/// if they meet the given permissions
	#[instrument(skip(conn))]
	pub async fn admin_or(
		p_id: i32,
		l_id: i32,
		other_auth: AuthorityPermissions,
		other_loc: LocationPermissions,
		conn: &DbConn,
	) -> Result<bool, Error> {
		let mut can_manage = false;

		let perm_includes =
			LocationIncludes { created_by: true, ..Default::default() };
		let (location, ..) =
			Location::get_by_id(l_id, perm_includes, conn).await?;

		#[allow(clippy::collapsible_if)]
		if let Some(Some(cr)) = location.created_by {
			if cr.id == p_id {
				can_manage = true;
			}
		}

		let (auth_perms, loc_perms) =
			Location::get_profile_permissions(l_id, p_id, conn).await?;

		#[allow(clippy::collapsible_if)]
		if let Some(auth_perms) = auth_perms {
			if auth_perms
				.intersects(AuthorityPermissions::Administrator | other_auth)
			{
				can_manage = true;
			} else {
				// If the given profile is the owner of the location but they
				// don't have the necessary permissions in this authority they
				// should not be allowed to manage this location
				can_manage = false;
			}
		}

		#[allow(clippy::collapsible_if)]
		if let Some(loc_perms) = loc_perms {
			if loc_perms
				.intersects(LocationPermissions::Administrator | other_loc)
			{
				can_manage = true;
			}
		}

		Ok(can_manage)
	}

	/// Get all the members of this location
	#[instrument(skip(conn))]
	pub async fn get_members(
		l_id: i32,
		conn: &DbConn,
	) -> Result<
		Vec<(PrimitiveProfile, Option<Image>, LocationPermissions)>,
		Error,
	> {
		let members = conn
			.interact(move |conn| {
				location_profile::table
					.filter(location_profile::location_id.eq(l_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(location_profile::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						location_profile::permissions,
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(prof, img, perm): (_, _, i64)| {
				let perm = LocationPermissions::from_bits_truncate(perm);
				(prof, img, perm)
			})
			.collect();

		Ok(members)
	}

	/// Delete a member from this location
	#[instrument(skip(conn))]
	pub async fn delete_member(
		loc_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::db::location_profile::dsl::*;

			diesel::delete(location_profile.find((loc_id, prof_id)))
				.execute(conn)
		})
		.await??;

		info!("deleted profile {prof_id} from location {loc_id}");

		Ok(())
	}
}

#[derive(Clone, Copy, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = location_profile)]
#[diesel(check_for_backend(Pg))]
pub struct NewLocationProfile {
	pub location_id: i32,
	pub profile_id:  i32,
	pub added_by:    i32,
	pub permissions: i64,
}

impl NewLocationProfile {
	/// Insert this [`NewLocationProfile`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		conn: &DbConn,
	) -> Result<(PrimitiveProfile, Option<Image>, LocationPermissions), Error>
	{
		conn.interact(move |conn| {
			use crate::db::location_profile::dsl::*;

			diesel::insert_into(location_profile).values(self).execute(conn)
		})
		.await??;

		let (profile, img, permissions): (_, _, i64) = conn
			.interact(move |conn| {
				location_profile::table
					.filter(
						location_profile::location_id.eq(self.location_id).and(
							location_profile::profile_id.eq(self.profile_id),
						),
					)
					.inner_join(
						profile::table
							.on(profile::id.eq(location_profile::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						location_profile::permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions = LocationPermissions::from_bits_truncate(permissions);

		info!(
			"added profile {} to location {}",
			self.profile_id, self.location_id
		);

		Ok((profile, img, permissions))
	}
}

#[derive(AsChangeset, Clone, Copy, Debug, Deserialize, Serialize)]
#[diesel(table_name = location_profile)]
#[diesel(check_for_backend(Pg))]
pub struct LocationProfileUpdate {
	pub updated_by:  i32,
	pub permissions: i64,
}

impl LocationProfileUpdate {
	/// Apply this update to the [`Location`] with the given id
	pub async fn apply_to(
		self,
		loc_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<(PrimitiveProfile, Option<Image>, LocationPermissions), Error>
	{
		conn.interact(move |conn| {
			use crate::db::location_profile::dsl::*;

			diesel::update(location_profile.find((loc_id, prof_id)))
				.set(self)
				.execute(conn)
		})
		.await??;

		let (profile, img, permissions): (_, _, i64) = conn
			.interact(move |conn| {
				location_profile::table
					.find((loc_id, prof_id))
					.inner_join(
						profile::table
							.on(profile::id.eq(location_profile::profile_id)),
					)
					.left_outer_join(
						image::table
							.on(profile::avatar_image_id
								.eq(image::id.nullable())),
					)
					.select((
						PrimitiveProfile::as_select(),
						image::all_columns.nullable(),
						location_profile::permissions,
					))
					.get_result(conn)
			})
			.await??;

		let permissions = LocationPermissions::from_bits_truncate(permissions);

		info!(
			"set permissions for profile {} to {} in location {}",
			prof_id, self.permissions, loc_id
		);

		Ok((profile, img, permissions))
	}
}
