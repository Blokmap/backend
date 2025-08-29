use common::{DbConn, Error};
use db::{
	CreatorAlias,
	UpdaterAlias,
	creator,
	location_role,
	profile,
	updater,
};
use diesel::dsl::{AliasedFields, Nullable};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use primitives::{PrimitiveLocationRole, PrimitiveProfile};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct LocationRoleIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct LocationRole {
	#[diesel(embed)]
	pub primitive:  PrimitiveLocationRole,
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

impl LocationRole {
	/// Build a query with all required (dynamic) joins to select a full
	/// location role data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: LocationRoleIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		location_role::table
			.left_join(
				creator.on(inc_created_by.into_sql::<Bool>().and(
					location_role::created_by
						.eq(creator.field(profile::id).nullable()),
				)),
			)
			.left_join(
				updater.on(inc_updated_by.into_sql::<Bool>().and(
					location_role::updated_by
						.eq(updater.field(profile::id).nullable()),
				)),
			)
	}

	/// Get a [`LocationRole`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		role_id: i32,
		includes: LocationRoleIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				query
					.filter(location_role::id.eq(role_id))
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
	) -> Result<PrimitiveLocationRole, Error> {
		let role = conn
			.interact(move |conn| {
				diesel::delete(location_role::table.find(r_id))
					.returning(PrimitiveLocationRole::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(role)
	}

	/// Get all [`LocationRole`]s for a given location
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		l_id: i32,
		includes: LocationRoleIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				query
					.filter(location_role::location_id.eq(l_id))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(role)
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = location_role)]
#[diesel(check_for_backend(Pg))]
pub struct NewLocationRole {
	pub location_id: i32,
	pub name:        String,
	pub permissions: i64,
	pub created_by:  i32,
}

impl NewLocationRole {
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		includes: LocationRoleIncludes,
		conn: &DbConn,
	) -> Result<LocationRole, Error> {
		let new_role_id = conn
			.interact(move |conn| {
				diesel::insert_into(location_role::table)
					.values(self)
					.returning(location_role::id)
					.get_result(conn)
			})
			.await??;

		let role = LocationRole::get_by_id(new_role_id, includes, conn).await?;

		Ok(role)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize)]
#[diesel(table_name = location_role)]
pub struct LocationRoleUpdate {
	pub name:        Option<String>,
	pub permissions: Option<i64>,
	pub updated_by:  i32,
}

impl LocationRoleUpdate {
	/// Update this [`LocationRole`] in the database.
	#[instrument(skip(conn))]
	pub async fn apply_to(
		self,
		role_id: i32,
		includes: LocationRoleIncludes,
		conn: &DbConn,
	) -> Result<LocationRole, Error> {
		let role_id = conn
			.interact(move |conn| {
				use self::location_role::dsl::*;

				diesel::update(location_role.find(role_id))
					.set(self)
					.returning(id)
					.get_result(conn)
			})
			.await??;

		let role = LocationRole::get_by_id(role_id, includes, conn).await?;

		info!("updated location role {role:?}");

		Ok(role)
	}
}
