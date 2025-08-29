#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate tracing;

use common::{DbConn, DbPool, Error, InternalServerError};
use db::{
	authority,
	authority_member,
	institution,
	institution_member,
	location,
	location_member,
	profile,
	role,
};
use diesel::prelude::*;
use serde::Serialize;

bitflags! {
	/// All possible permissions
	#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
	pub struct Permissions: i64 {
		/// Institution admin, member can do everything
		const InstAdministrator = 1 << 0;
		/// Member can create or link new authorities for this institution
		const InstAddAuthority = 1 << 1;
		/// Member can delete authorities for this institution
		const InstDeleteAuthority = 1 << 2;
		/// Member can manage institution members:
		/// - add members
		/// - update member roles
		/// - remove members
		const InstManageMembers = 1 << 3;

		/// Admin privileges, member can do everything
		const AuthAdministrator = 1 << 4;
		/// Member can submit new locations
		const AuthAddLocations = 1 << 5;
		/// Member can approve/reject all locations under this authority
		const AuthApproveLocations = 1 << 6;
		/// Member can delete all locations under this authority
		const AuthDeleteLocations = 1 << 7;
		/// Member can manage authority members:
		/// - add members
		/// - update member roles
		/// - remove members
		const AuthManageMembers = 1 << 8;

		/// Admin privileges, member can do everything
		const LocAdministrator = 1 << 9;
		/// Member can manage images for this location:
		/// - upload new images
		/// - reorder images
		/// - delete images
		const LocManageImages = 1 << 10;
		/// Member can manage opening times for this location:
		/// - create opening times
		/// - update opening times
		/// - delete opening times
		const LocManageOpeningTimes = 1 << 11;
		/// Member can manage location members
		/// - add members
		/// - update member roles
		/// - remove members
		const LocManageMembers = 1 << 12;
		/// Member can confirm reservations for this location:
		const LocConfirmReservations = 1 << 13;
	}
}

impl Permissions {
	#[instrument(skip(conn))]
	pub async fn get_for_institution_member(
		inst_id: i32,
		prof_id: i32,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let perms: Option<i64> = conn
			.interact(move |conn| {
				institution::table
					.find(inst_id)
					.inner_join(
						institution_member::table.on(
							institution_member::institution_id.eq(inst_id).and(
								institution_member::profile_id.eq(prof_id),
							),
						),
					)
					.inner_join(role::table.on(
						institution_member::role_id.eq(role::id.nullable()),
					))
					.select(role::permissions)
					.get_result(conn)
					.optional()
			})
			.await??;

		let perms = perms.unwrap_or_default();
		let perms = Self::from_bits_truncate(perms);

		Ok(perms)
	}

	#[instrument(skip(conn))]
	async fn get_for_authority_member_inner(
		auth_id: i32,
		prof_id: i32,
		conn: DbConn,
	) -> Result<Self, Error> {
		let perms: Option<i64> = conn
			.interact(move |conn| {
				authority::table
					.find(auth_id)
					.inner_join(
						authority_member::table.on(
							authority_member::authority_id
								.eq(auth_id)
								.and(authority_member::profile_id.eq(prof_id)),
						),
					)
					.inner_join(
						role::table
							.on(authority_member::role_id
								.eq(role::id.nullable())),
					)
					.select(role::permissions)
					.get_result(conn)
					.optional()
			})
			.await??;

		let perms = perms.unwrap_or_default();
		let perms = Self::from_bits_truncate(perms);

		Ok(perms)
	}

	#[instrument(skip(pool))]
	pub async fn get_for_authority_member(
		auth_id: i32,
		prof_id: i32,
		pool: &DbPool,
	) -> Result<Self, Error> {
		let auth_conn = pool.get().await?;
		let auth_handle = tokio::spawn(Self::get_for_authority_member_inner(
			auth_id, prof_id, auth_conn,
		));

		let inst_conn = pool.get().await?;
		let inst_id = inst_conn
			.interact(move |conn| {
				authority::table
					.find(auth_id)
					.select(authority::institution_id.nullable())
					.get_result(conn)
			})
			.await??;

		let i_perms = if let Some(inst_id) = inst_id {
			let inst_conn = pool.get().await?;
			Self::get_for_institution_member(inst_id, prof_id, &inst_conn)
				.await?
		} else {
			Self::empty()
		};

		let a_perms =
			auth_handle.await.map_err(InternalServerError::JoinError)??;

		Ok(a_perms | i_perms)
	}

	#[instrument(skip(conn))]
	async fn get_for_location_member_inner(
		loc_id: i32,
		prof_id: i32,
		conn: DbConn,
	) -> Result<Self, Error> {
		let perms: Option<i64> = conn
			.interact(move |conn| {
				location::table
					.find(loc_id)
					.inner_join(
						location_member::table.on(location_member::location_id
							.eq(loc_id)
							.and(location_member::profile_id.eq(prof_id))),
					)
					.inner_join(
						role::table
							.on(location_member::role_id
								.eq(role::id.nullable())),
					)
					.select(role::permissions)
					.get_result(conn)
					.optional()
			})
			.await??;

		let perms = perms.unwrap_or_default();
		let perms = Self::from_bits_truncate(perms);

		Ok(perms)
	}

	#[instrument(skip(pool))]
	pub async fn get_for_location_member(
		loc_id: i32,
		prof_id: i32,
		pool: &DbPool,
	) -> Result<Self, Error> {
		let loc_conn = pool.get().await?;
		let loc_handle = tokio::spawn(Self::get_for_location_member_inner(
			loc_id, prof_id, loc_conn,
		));

		let auth_conn = pool.get().await?;
		let auth_id = auth_conn
			.interact(move |conn| {
				location::table
					.find(loc_id)
					.select(location::authority_id.nullable())
					.get_result(conn)
			})
			.await??;

		let a_perms = if let Some(auth_id) = auth_id {
			Self::get_for_authority_member(auth_id, prof_id, pool).await?
		} else {
			Self::empty()
		};

		let l_perms =
			loc_handle.await.map_err(InternalServerError::JoinError)??;

		Ok(l_perms | a_perms)
	}

	/// Checks whether the given profile has *any* of the specified permissions
	/// for the given institution
	///
	/// Also returns `Ok(())` if the profile is a global admin
	#[instrument(skip(pool))]
	pub async fn check_for_institution(
		inst_id: i32,
		prof_id: i32,
		perms: Self,
		pool: &DbPool,
	) -> Result<(), Error> {
		let profile_conn = pool.get().await?;
		let profile_fetch = profile_conn.interact(move |conn| {
			profile::table
				.find(prof_id)
				.select(profile::is_admin)
				.get_result(conn)
		});

		let inst_conn = pool.get().await?;
		let (inst_perms, is_admin) = tokio::join!(
			Self::get_for_institution_member(inst_id, prof_id, &inst_conn),
			profile_fetch,
		);

		let inst_perms = inst_perms?;
		let is_admin: bool = is_admin??;

		if is_admin {
			return Ok(());
		}

		if inst_perms.intersects(perms) {
			return Ok(());
		}

		Err(Error::Forbidden)
	}

	/// Checks whether the given profile has *any* of the specified permissions
	/// for the given authority
	///
	/// Also returns `Ok(())` if the profile is a global admin
	#[instrument(skip(pool))]
	pub async fn check_for_authority(
		auth_id: i32,
		prof_id: i32,
		perms: Self,
		pool: &DbPool,
	) -> Result<(), Error> {
		let profile_conn = pool.get().await?;
		let profile_fetch = profile_conn.interact(move |conn| {
			profile::table
				.find(prof_id)
				.select(profile::is_admin)
				.get_result(conn)
		});

		let (auth_perms, is_admin) = tokio::join!(
			Self::get_for_authority_member(auth_id, prof_id, pool),
			profile_fetch,
		);

		let auth_perms = auth_perms?;
		let is_admin: bool = is_admin??;

		if is_admin {
			return Ok(());
		}

		if auth_perms.intersects(perms) {
			return Ok(());
		}

		Err(Error::Forbidden)
	}

	/// Checks whether the given profile has *any* of the specified permissions
	/// for the given location
	///
	/// Also returns `Ok(())` if the profile is a global admin
	#[instrument(skip(pool))]
	pub async fn check_for_location(
		loc_id: i32,
		prof_id: i32,
		perms: Self,
		pool: &DbPool,
	) -> Result<(), Error> {
		let admin_conn = pool.get().await?;
		let admin_fetch = admin_conn.interact(move |conn| {
			profile::table
				.find(prof_id)
				.select(profile::is_admin)
				.get_result(conn)
		});

		let (loc_perms, is_admin) = tokio::join!(
			Self::get_for_location_member(loc_id, prof_id, pool),
			admin_fetch,
		);

		let loc_perms = loc_perms?;
		let is_admin: bool = is_admin??;

		if is_admin {
			return Ok(());
		}

		if loc_perms.intersects(perms) {
			return Ok(());
		}

		Err(Error::Forbidden)
	}
}
