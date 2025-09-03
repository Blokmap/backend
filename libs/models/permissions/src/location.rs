use common::{DbPool, Error};
use db::{location, location_member, location_role};
use diesel::prelude::*;
use serde::Serialize;

use crate::{AuthorityPermissions, InstitutionPermissions};

bitflags! {
	/// All possible permissions
	#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
	pub struct LocationPermissions: i64 {
		/// Admin privileges, member can do everything
		const Administrator = 1 << 9;
		/// Member can manage images for this location:
		/// - upload new images
		/// - reorder images
		/// - delete images
		const ManageImages = 1 << 10;
		/// Member can manage opening times for this location:
		/// - create opening times
		/// - update opening times
		/// - delete opening times
		const ManageOpeningTimes = 1 << 11;
		/// Member can manage location members
		/// - add members
		/// - update member roles
		/// - remove members
		const ManageMembers = 1 << 12;
		/// Member can confirm reservations for this location:
		const ConfirmReservations = 1 << 13;
	}
}

impl LocationPermissions {
	#[instrument(skip(pool))]
	pub(crate) async fn get_for_location_member(
		loc_id: i32,
		prof_id: i32,
		pool: &DbPool,
	) -> Result<(InstitutionPermissions, AuthorityPermissions, Self), Error> {
		let ia_conn = pool.get().await?;
		let ia_perms_future = async {
			let auth_id = ia_conn
				.interact(move |conn| {
					use self::location::dsl::*;

					location.find(loc_id).select(authority_id).get_result(conn)
				})
				.await??;

			let Some(auth_id) = auth_id else {
				return Ok::<_, Error>((
					InstitutionPermissions::empty(),
					AuthorityPermissions::empty(),
				));
			};

			let ia_perms = AuthorityPermissions::get_for_authority_member(
				auth_id, prof_id, pool,
			)
			.await?;

			Ok(ia_perms)
		};

		let loc_conn = pool.get().await?;
		let loc_perms_future = async {
			let loc_perms = loc_conn
				.interact(move |conn| {
					use self::location_member::dsl::*;

					location_member
						.filter(
							location_id.eq(loc_id).and(profile_id.eq(prof_id)),
						)
						.inner_join(location_role::table.on(
							location_role_id.eq(location_role::id.nullable()),
						))
						.select(location_role::permissions)
						.get_result(conn)
						.optional()
				})
				.await??;

			let loc_perms = loc_perms.unwrap_or_default();
			let loc_perms = Self::from_bits_truncate(loc_perms);

			Ok::<_, Error>(loc_perms)
		};

		let (ia_perms, loc_perms) =
			tokio::join!(ia_perms_future, loc_perms_future,);

		let (inst_perms, auth_perms) = ia_perms?;
		let loc_perms = loc_perms?;

		Ok((inst_perms, auth_perms, loc_perms))
	}
}
