#[macro_use]
extern crate tracing;

use common::{DbConn, Error};
use db::{
	CreatorAlias,
	UpdaterAlias,
	authority_role,
	creator,
	institution_role,
	location_role,
	profile,
	role,
	updater,
};
use diesel::dsl::{AliasedFields, Nullable};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use primitives::{PrimitiveProfile, PrimitiveRole};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct RoleIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct Role {
	#[diesel(embed)]
	pub primitive:  PrimitiveRole,
	#[diesel(select_expression = created_by_fragment())]
	pub created_by: Option<PrimitiveProfile>,
	#[diesel(select_expression = updated_by_fragment())]
	pub updated_by: Option<PrimitiveProfile>,
}

#[allow(non_camel_case_types)]
type created_by_fragment = Nullable<
	AliasedFields<CreatorAlias, <profile::table as Table>::AllColumns>,
>;
fn created_by_fragment() -> created_by_fragment {
	creator.fields(profile::all_columns).nullable()
}

#[allow(non_camel_case_types)]
type updated_by_fragment = Nullable<
	AliasedFields<UpdaterAlias, <profile::table as Table>::AllColumns>,
>;
fn updated_by_fragment() -> updated_by_fragment {
	updater.fields(profile::all_columns).nullable()
}

impl Role {
	/// Build a query with all required (dynamic) joins to select a full
	/// location role data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: RoleIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		role::table
			.left_join(creator.on(inc_created_by.into_sql::<Bool>().and(
				role::created_by.eq(creator.field(profile::id).nullable()),
			)))
			.left_join(updater.on(inc_updated_by.into_sql::<Bool>().and(
				role::updated_by.eq(updater.field(profile::id).nullable()),
			)))
	}

	/// Get a [`LocationRole`] given its id
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
					.filter(role::id.eq(role_id))
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		Ok(role)
	}

	/// Delete a [`LocationRole`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(
		r_id: i32,
		conn: &DbConn,
	) -> Result<PrimitiveRole, Error> {
		let role = conn
			.interact(move |conn| {
				diesel::delete(role::table.find(r_id))
					.returning(PrimitiveRole::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(role)
	}

	/// Get all [`Role`]s for a given institution
	#[instrument(skip(conn))]
	pub async fn get_for_institution(
		l_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				institution_role::table
					.filter(institution_role::institution_id.eq(l_id))
					.inner_join(
						query.on(institution_role::role_id.eq(role::id)),
					)
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(role)
	}

	/// Get all [`Role`]s for a given authority
	#[instrument(skip(conn))]
	pub async fn get_for_authority(
		a_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				authority_role::table
					.filter(authority_role::authority_id.eq(a_id))
					.inner_join(query.on(authority_role::role_id.eq(role::id)))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(role)
	}

	/// Get all [`Role`]s for a given location
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		l_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				location_role::table
					.filter(location_role::location_id.eq(l_id))
					.inner_join(query.on(location_role::role_id.eq(role::id)))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(role)
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = role)]
#[diesel(check_for_backend(Pg))]
pub struct NewRole {
	pub name:        String,
	pub colour:      Option<String>,
	pub permissions: i64,
	pub created_by:  i32,
}

impl NewRole {
	#[instrument(skip(conn))]
	pub async fn insert_for_institution(
		self,
		inst_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Role, Error> {
		let new_role_id = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					let new_role_id = diesel::insert_into(role::table)
						.values(self)
						.returning(role::id)
						.get_result(conn)?;

					diesel::insert_into(institution_role::table)
						.values((
							institution_role::institution_id.eq(inst_id),
							institution_role::role_id.eq(new_role_id),
						))
						.execute(conn)?;

					Ok(new_role_id)
				})
			})
			.await??;

		let role = Role::get_by_id(new_role_id, includes, conn).await?;

		Ok(role)
	}

	#[instrument(skip(conn))]
	pub async fn insert_for_authority(
		self,
		auth_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Role, Error> {
		let new_role_id = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					let new_role_id = diesel::insert_into(role::table)
						.values(self)
						.returning(role::id)
						.get_result(conn)?;

					diesel::insert_into(authority_role::table)
						.values((
							authority_role::authority_id.eq(auth_id),
							authority_role::role_id.eq(new_role_id),
						))
						.execute(conn)?;

					Ok(new_role_id)
				})
			})
			.await??;

		let role = Role::get_by_id(new_role_id, includes, conn).await?;

		Ok(role)
	}

	#[instrument(skip(conn))]
	pub async fn insert_for_location(
		self,
		loc_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Role, Error> {
		let new_role_id = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					let new_role_id = diesel::insert_into(role::table)
						.values(self)
						.returning(role::id)
						.get_result(conn)?;

					diesel::insert_into(location_role::table)
						.values((
							location_role::location_id.eq(loc_id),
							location_role::role_id.eq(new_role_id),
						))
						.execute(conn)?;

					Ok(new_role_id)
				})
			})
			.await??;

		let role = Role::get_by_id(new_role_id, includes, conn).await?;

		Ok(role)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize)]
#[diesel(table_name = role)]
pub struct RoleUpdate {
	pub name:        Option<String>,
	pub colour:      Option<String>,
	pub permissions: Option<i64>,
	pub updated_by:  i32,
}

impl RoleUpdate {
	/// Update this [`Role`] in the database.
	#[instrument(skip(conn))]
	pub async fn apply_to(
		self,
		role_id: i32,
		includes: RoleIncludes,
		conn: &DbConn,
	) -> Result<Role, Error> {
		let role_id = conn
			.interact(move |conn| {
				use self::role::dsl::*;

				diesel::update(role.find(role_id))
					.set(self)
					.returning(id)
					.get_result(conn)
			})
			.await??;

		let role = Role::get_by_id(role_id, includes, conn).await?;

		Ok(role)
	}
}
