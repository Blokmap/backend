use common::{DbConn, Error};
use db::{
	CreatorAlias,
	UpdaterAlias,
	creator,
	institution_role,
	profile,
	updater,
};
use diesel::dsl::{AliasedFields, Nullable};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use primitives::{PrimitiveInstitutionRole, PrimitiveProfile};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct InstitutionRoleIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

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

impl InstitutionRole {
	/// Build a query with all required (dynamic) joins to select a full
	/// institution role data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: InstitutionRoleIncludes) -> _ {
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
		includes: InstitutionRoleIncludes,
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

	/// Get all [`InstitutionRole`]s for a given institution
	#[instrument(skip(conn))]
	pub async fn get_for_institution(
		i_id: i32,
		includes: InstitutionRoleIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let role = conn
			.interact(move |conn| {
				query
					.filter(institution_role::institution_id.eq(i_id))
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
	pub permissions:    i64,
	pub created_by:     i32,
}

impl NewInstitutionRole {
	#[instrument(skip(conn))]
	pub async fn insert(self, conn: &DbConn) -> Result<InstitutionRole, Error> {
		let new_role_id = conn
			.interact(move |conn| {
				diesel::insert_into(institution_role::table)
					.values(self)
					.returning(institution_role::id)
					.get_result(conn)
			})
			.await??;

		let role = InstitutionRole::get_by_id(
			new_role_id,
			InstitutionRoleIncludes::default(),
			conn,
		)
		.await?;

		Ok(role)
	}
}
