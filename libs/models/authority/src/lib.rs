#[macro_use]
extern crate tracing;

use ::role::NewAuthorityRole;
use common::{DbConn, Error};
use db::{
	CreatorAlias,
	UpdaterAlias,
	authority,
	authority_member,
	authority_role,
	creator,
	institution,
	profile,
	updater,
};
use diesel::dsl::{AliasedFields, Nullable};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use permissions::AuthorityPermissions;
use primitives::{PrimitiveAuthority, PrimitiveInstitution, PrimitiveProfile};
use serde::{Deserialize, Serialize};

mod member;

pub use member::*;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct AuthorityIncludes {
	#[serde(default)]
	pub created_by:  bool,
	#[serde(default)]
	pub updated_by:  bool,
	#[serde(default)]
	pub institution: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct Authority {
	#[diesel(embed)]
	pub primitive:   PrimitiveAuthority,
	#[diesel(select_expression = created_by_fragment())]
	pub created_by:  Option<PrimitiveProfile>,
	#[diesel(select_expression = updated_by_fragment())]
	pub updated_by:  Option<PrimitiveProfile>,
	#[diesel(embed)]
	pub institution: Option<PrimitiveInstitution>,
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

impl Authority {
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: AuthorityIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;
		let inc_institution: bool = includes.institution;

		authority::table
			.left_join(creator.on(inc_created_by.into_sql::<Bool>().and(
				authority::created_by.eq(creator.field(profile::id).nullable()),
			)))
			.left_join(updater.on(inc_updated_by.into_sql::<Bool>().and(
				authority::updated_by.eq(updater.field(profile::id).nullable()),
			)))
			.left_join(institution::table.on(
				inc_institution.into_sql::<Bool>().and(
					authority::institution_id.eq(institution::id.nullable()),
				),
			))
	}

	/// Get a single [`Authority`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		auth_id: i32,
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::query(includes);

		let authority = conn
			.interact(move |conn| {
				query
					.filter(authority::id.eq(auth_id))
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		// let authority = parts.join(includes);

		Ok(authority)
	}

	/// Get all [`Authorities`]s from the database, optionally including related
	/// profiles.
	#[instrument(skip(conn))]
	pub async fn get_all(
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::query(includes);

		let authorities = conn
			.interact(move |c| query.select(Self::as_select()).load(c))
			.await??;

		Ok(authorities)
	}

	/// Delete an [`Authority`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(
		auth_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::authority::dsl::*;

			diesel::delete(authority.find(auth_id)).execute(conn)
		})
		.await??;

		info!("deleted authority with id {auth_id}");

		Ok(())
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = authority)]
#[diesel(check_for_backend(Pg))]
pub struct NewAuthority {
	pub name:           String,
	pub description:    Option<String>,
	pub created_by:     i32,
	pub institution_id: Option<i32>,
}

impl NewAuthority {
	/// Insert this [`NewAuthority`]
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Authority, Error> {
		let authority = conn
			.interact(|conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use self::authority::dsl::*;

					let creator_id = self.created_by;

					let auth = diesel::insert_into(authority)
						.values(self)
						.returning(PrimitiveAuthority::as_returning())
						.get_result(conn)?;

					let new_role = NewAuthorityRole {
						authority_id: auth.id,
						name:         "owner".into(),
						colour:       None,
						permissions:  AuthorityPermissions::Administrator
							.bits(),
						created_by:   creator_id,
					};

					let role_id = diesel::insert_into(authority_role::table)
						.values(new_role)
						.returning(authority_role::id)
						.get_result(conn)?;

					let member = NewAuthorityMember {
						authority_id:      auth.id,
						profile_id:        creator_id,
						authority_role_id: Some(role_id),
						added_by:          creator_id,
					};

					diesel::insert_into(authority_member::table)
						.values(member)
						.execute(conn)?;

					Ok(auth)
				})
			})
			.await??;

		let authority =
			Authority::get_by_id(authority.id, includes, conn).await?;

		info!("created authority {authority:?}");

		Ok(authority)
	}
}

#[derive(AsChangeset, Clone, Debug, Deserialize, Serialize)]
#[diesel(table_name = authority)]
#[diesel(check_for_backend(Pg))]
pub struct AuthorityUpdate {
	pub name:           Option<String>,
	pub description:    Option<String>,
	pub updated_by:     i32,
	pub institution_id: Option<i32>,
}

impl AuthorityUpdate {
	/// Apply this update to the [`Authority`] with the given id
	pub async fn apply_to(
		self,
		auth_id: i32,
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Authority, Error> {
		conn.interact(move |conn| {
			use self::authority::dsl::*;

			diesel::update(authority.find(auth_id)).set(self).execute(conn)
		})
		.await??;

		let authority = Authority::get_by_id(auth_id, includes, conn).await?;

		info!("updated authority {authority:?}");

		Ok(authority)
	}
}
