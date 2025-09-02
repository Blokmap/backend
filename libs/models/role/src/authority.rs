use common::{DbConn, Error};
use db::{authority_role, creator, profile, updater};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use primitives::{PrimitiveAuthorityRole, PrimitiveProfile};
use serde::{Deserialize, Serialize};

use crate::{RoleIncludes, created_by_fragment, updated_by_fragment};

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct AuthorityRole {
	#[diesel(embed)]
	pub primitive:  PrimitiveAuthorityRole,
	#[diesel(select_expression = created_by_fragment())]
	pub created_by: Option<PrimitiveProfile>,
	#[diesel(select_expression = updated_by_fragment())]
	pub updated_by: Option<PrimitiveProfile>,
}

impl AuthorityRole {
	/// Build a query with all required (dynamic) joins to select a full
	/// location role data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: RoleIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		authority_role::table
			.left_join(
				creator.on(inc_created_by.into_sql::<Bool>().and(
					authority_role::created_by
						.eq(creator.field(profile::id).nullable()),
				)),
			)
			.left_join(
				updater.on(inc_updated_by.into_sql::<Bool>().and(
					authority_role::updated_by
						.eq(updater.field(profile::id).nullable()),
				)),
			)
	}

	/// Get a [`AuthorityRole`] given its id
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
					.filter(authority_role::id.eq(role_id))
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		Ok(role)
	}

	/// Delete a [`AuthorityRole`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(
		r_id: i32,
		conn: &DbConn,
	) -> Result<PrimitiveAuthorityRole, Error> {
		let role = conn
			.interact(move |conn| {
				diesel::delete(authority_role::table.find(r_id))
					.returning(PrimitiveAuthorityRole::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(role)
	}

	/// Get all [`AuthorityRole`]s for a given location
	#[instrument(skip(conn))]
	pub async fn get_for_authority(
		auth_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				use self::authority_role::dsl::*;

				query
					.filter(authority_id.eq(auth_id))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(role)
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = authority_role)]
#[diesel(check_for_backend(Pg))]
pub struct NewAuthorityRole {
	pub authority_id: i32,
	pub name:         String,
	pub colour:       Option<String>,
	pub permissions:  i64,
	pub created_by:   i32,
}

impl NewAuthorityRole {
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		loc_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<AuthorityRole, Error> {
		let new_role_id = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					let new_role_id =
						diesel::insert_into(authority_role::table)
							.values(self)
							.returning(authority_role::id)
							.get_result(conn)?;

					Ok(new_role_id)
				})
			})
			.await??;

		let role =
			AuthorityRole::get_by_id(new_role_id, includes, conn).await?;

		Ok(role)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize)]
#[diesel(table_name = authority_role)]
pub struct AuthorityRoleUpdate {
	pub name:        Option<String>,
	pub colour:      Option<String>,
	pub permissions: Option<i64>,
	pub updated_by:  i32,
}

impl AuthorityRoleUpdate {
	/// Update this [`AuthorityRole`] in the database.
	#[instrument(skip(conn))]
	pub async fn apply_to(
		self,
		role_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<AuthorityRole, Error> {
		let role_id = conn
			.interact(move |conn| {
				use self::authority_role::dsl::*;

				diesel::update(authority_role.find(role_id))
					.set(self)
					.returning(id)
					.get_result(conn)
			})
			.await??;

		let role = AuthorityRole::get_by_id(role_id, includes, conn).await?;

		Ok(role)
	}
}
