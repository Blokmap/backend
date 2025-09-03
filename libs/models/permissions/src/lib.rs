#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate tracing;

use common::{DbConn, DbPool, Error};

mod authority;
mod institution;
mod location;

pub use authority::*;
pub use institution::*;
pub use location::*;

/// Checks whether the given profile has *any* of the specified permissions
/// for the given institution
#[instrument(skip(conn))]
pub async fn check_institution_perms(
	inst_id: i32,
	prof_id: i32,
	perms: InstitutionPermissions,
	conn: &DbConn,
) -> Result<(), Error> {
	let inst_perms = InstitutionPermissions::get_for_institution_member(
		inst_id, prof_id, conn,
	)
	.await?;

	if inst_perms.intersects(perms) {
		return Ok(());
	}

	Err(Error::Forbidden)
}

/// Checks whether the given profile has *any* of the specified permissions
/// for the given authority
#[instrument(skip(pool))]
pub async fn check_authority_perms(
	auth_id: i32,
	prof_id: i32,
	auth_perms: AuthorityPermissions,
	inst_perms: InstitutionPermissions,
	pool: &DbPool,
) -> Result<(), Error> {
	let (db_inst_perms, db_auth_perms) =
		AuthorityPermissions::get_for_authority_member(auth_id, prof_id, pool)
			.await?;

	if db_inst_perms.intersects(inst_perms)
		| db_auth_perms.intersects(auth_perms)
	{
		return Ok(());
	}

	Err(Error::Forbidden)
}

/// Checks whether the given profile has *any* of the specified permissions
/// for the given location
#[instrument(skip(pool))]
pub async fn check_location_perms(
	loc_id: i32,
	prof_id: i32,
	loc_perms: LocationPermissions,
	auth_perms: AuthorityPermissions,
	inst_perms: InstitutionPermissions,
	pool: &DbPool,
) -> Result<(), Error> {
	let (db_inst_perms, db_auth_perms, db_loc_perms) =
		LocationPermissions::get_for_location_member(loc_id, prof_id, pool)
			.await?;

	if db_inst_perms.intersects(inst_perms)
		| db_auth_perms.intersects(auth_perms)
		| db_loc_perms.intersects(loc_perms)
	{
		return Ok(());
	}

	Err(Error::Forbidden)
}
