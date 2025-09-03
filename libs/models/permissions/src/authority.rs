use common::{DbPool, Error};
use db::{authority, authority_member, authority_role};
use diesel::prelude::*;
use serde::Serialize;

use crate::InstitutionPermissions;

bitflags! {
	/// All possible permissions
	#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
	pub struct AuthorityPermissions: i64 {
		/// Admin privileges, member can do everything
		const Administrator = 1 << 4;
		/// Member can submit new locations
		const AddLocations = 1 << 5;
		/// Member can approve/reject all locations under this authority
		const ApproveLocations = 1 << 6;
		/// Member can delete all locations under this authority
		const DeleteLocations = 1 << 7;
		/// Member can manage authority members:
		/// - add members
		/// - update member roles
		/// - remove members
		const ManageMembers = 1 << 8;
	}
}

impl AuthorityPermissions {
	#[instrument(skip(pool))]
	pub(crate) async fn get_for_authority_member(
		auth_id: i32,
		prof_id: i32,
		pool: &DbPool,
	) -> Result<(InstitutionPermissions, Self), Error> {
		let inst_conn = pool.get().await?;
		let inst_perms_future = async {
			let inst_id = inst_conn
				.interact(move |conn| {
					use self::authority::dsl::*;

					authority
						.find(auth_id)
						.select(institution_id)
						.get_result(conn)
				})
				.await??;

			let Some(inst_id) = inst_id else {
				return Ok::<_, Error>(InstitutionPermissions::empty());
			};

			let inst_perms =
				InstitutionPermissions::get_for_institution_member(
					inst_id, prof_id, &inst_conn,
				)
				.await?;

			Ok(inst_perms)
		};

		let auth_conn = pool.get().await?;
		let auth_perms_future = async {
			let auth_perms = auth_conn
				.interact(move |conn| {
					use self::authority_member::dsl::*;

					authority_member
						.filter(
							authority_id
								.eq(auth_id)
								.and(profile_id.eq(prof_id)),
						)
						.inner_join(authority_role::table.on(
							authority_role_id.eq(authority_role::id.nullable()),
						))
						.select(authority_role::permissions)
						.get_result(conn)
						.optional()
				})
				.await??;

			let auth_perms = auth_perms.unwrap_or_default();
			let auth_perms = Self::from_bits_truncate(auth_perms);

			Ok::<_, Error>(auth_perms)
		};

		let (inst_perms, auth_perms) =
			tokio::join!(inst_perms_future, auth_perms_future,);

		let inst_perms = inst_perms?;
		let auth_perms = auth_perms?;

		Ok((inst_perms, auth_perms))
	}
}
