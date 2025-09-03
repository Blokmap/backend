use common::{DbConn, Error};
use db::{creator, institution_role, profile, updater};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use primitives::{PrimitiveInstitutionRole, PrimitiveProfile};
use serde::{Deserialize, Serialize};

use crate::{RoleIncludes, created_by_fragment, updated_by_fragment};

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct InstitutionRole {
	#[diesel(embed)]
	pub primitive:  PrimitiveInstitutionRole,
	#[diesel(select_expression = created_by_fragment())]
	pub created_by: Option<PrimitiveProfile>,
	#[diesel(select_expression = updated_by_fragment())]
	pub updated_by: Option<PrimitiveProfile>,
}

impl InstitutionRole {
	/// Build a query with all required (dynamic) joins to select a full
	/// location role data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: RoleIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		institution_role::table
			.left_join(
				creator.on(inc_created_by.into_sql::<Bool>().and(
					institution_role::created_by
						.eq(creator.field(profile::id).nullable()),
				)),
			)
			.left_join(
				updater.on(inc_updated_by.into_sql::<Bool>().and(
					institution_role::updated_by
						.eq(updater.field(profile::id).nullable()),
				)),
			)
	}

	/// Get a [`InstitutionRole`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		role_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				query
					.filter(institution_role::id.eq(role_id))
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		Ok(role)
	}

	/// Delete a [`InstitutionRole`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(
		r_id: i32,
		conn: &DbConn,
	) -> Result<PrimitiveInstitutionRole, Error> {
		let role = conn
			.interact(move |conn| {
				diesel::delete(institution_role::table.find(r_id))
					.returning(PrimitiveInstitutionRole::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(role)
	}

	/// Get all [`InstitutionRole`]s for a given location
	#[instrument(skip(conn))]
	pub async fn get_for_institution(
		inst_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				use self::institution_role::dsl::*;

				query
					.filter(institution_id.eq(inst_id))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(role)
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = institution_role)]
#[diesel(check_for_backend(Pg))]
pub struct NewInstitutionRole {
	pub institution_id: i32,
	pub name:           String,
	pub colour:         Option<String>,
	pub permissions:    i64,
	pub created_by:     i32,
}

impl NewInstitutionRole {
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		loc_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<InstitutionRole, Error> {
		let new_role_id = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					let new_role_id =
						diesel::insert_into(institution_role::table)
							.values(self)
							.returning(institution_role::id)
							.get_result(conn)?;

					Ok(new_role_id)
				})
			})
			.await??;

		let role =
			InstitutionRole::get_by_id(new_role_id, includes, conn).await?;

		Ok(role)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize)]
#[diesel(table_name = institution_role)]
pub struct InstitutionRoleUpdate {
	pub name:        Option<String>,
	pub colour:      Option<String>,
	pub permissions: Option<i64>,
	pub updated_by:  i32,
}

impl InstitutionRoleUpdate {
	/// Update this [`InstitutionRole`] in the database.
	#[instrument(skip(conn))]
	pub async fn apply_to(
		self,
		role_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<InstitutionRole, Error> {
		let role_id = conn
			.interact(move |conn| {
				use self::institution_role::dsl::*;

				diesel::update(institution_role.find(role_id))
					.set(self)
					.returning(id)
					.get_result(conn)
			})
			.await??;

		let role = InstitutionRole::get_by_id(role_id, includes, conn).await?;

		Ok(role)
	}
}
