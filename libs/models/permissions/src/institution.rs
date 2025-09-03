use common::{DbConn, Error};
use db::{institution_member, institution_role};
use diesel::prelude::*;
use serde::Serialize;

bitflags! {
	/// All possible permissions
	#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
	pub struct InstitutionPermissions: i64 {
		/// Institution admin, member can do everything
		const Administrator = 1 << 0;
		/// Member can create or link new authorities for this institution
		const AddAuthority = 1 << 1;
		/// Member can delete authorities for this institution
		const DeleteAuthority = 1 << 2;
		/// Member can manage institution members:
		/// - add members
		/// - update member roles
		/// - remove members
		const ManageMembers = 1 << 3;
	}
}

impl InstitutionPermissions {
	#[instrument(skip(conn))]
	pub(crate) async fn get_for_institution_member(
		inst_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let perms: Option<i64> = conn
			.interact(move |conn| {
				use self::institution_member::dsl::*;

				institution_member
					.filter(
						institution_id.eq(inst_id).and(profile_id.eq(prof_id)),
					)
					.inner_join(institution_role::table.on(
						institution_role_id.eq(institution_role::id.nullable()),
					))
					.select(institution_role::permissions)
					.get_result(conn)
					.optional()
			})
			.await??;

		let perms = perms.unwrap_or_default();
		let perms = Self::from_bits_truncate(perms);

		Ok(perms)
	}
}
