use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};

use crate::PrimitiveProfile;
use crate::db::{authority, creator, profile, updater};

mod member;

pub use member::*;

pub type JoinedAuthorityData =
	(PrimitiveAuthority, Option<PrimitiveProfile>, Option<PrimitiveProfile>);

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct AuthorityIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(table_name = authority)]
#[diesel(check_for_backend(Pg))]
pub struct Authority {
	pub authority:  PrimitiveAuthority,
	pub created_by: Option<Option<PrimitiveProfile>>,
	pub updated_by: Option<Option<PrimitiveProfile>>,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = authority)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveAuthority {
	pub id:          i32,
	pub name:        String,
	pub description: Option<String>,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<i32>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<i32>,
}

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl Authority {
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn joined_query(includes: AuthorityIncludes) -> _ {
		let inc_created_by: bool = includes.created_by;
		let inc_updated_by: bool = includes.updated_by;

		authority::table
			.left_outer_join(creator.on(inc_created_by.into_sql::<Bool>().and(
				authority::created_by.eq(creator.field(profile::id).nullable()),
			)))
			.left_outer_join(updater.on(inc_updated_by.into_sql::<Bool>().and(
				authority::updated_by.eq(updater.field(profile::id).nullable()),
			)))
	}

	/// Construct a full [`Authority`] struct from the data returned by a
	/// joined query
	fn from_joined(
		includes: AuthorityIncludes,
		data: JoinedAuthorityData,
	) -> Self {
		Self {
			authority:  data.0,
			created_by: if includes.created_by { Some(data.1) } else { None },
			updated_by: if includes.updated_by { Some(data.2) } else { None },
		}
	}

	/// Get a single [`Authority`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		auth_id: i32,
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query(includes);

		let authority = conn
			.interact(move |conn| {
				query
					.filter(authority::id.eq(auth_id))
					.select((
						PrimitiveAuthority::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let authority = Self::from_joined(includes, authority);

		Ok(authority)
	}

	/// Get all [`Authorities`]s from the database, optionally including related
	/// profiles.
	#[instrument(skip(conn))]
	pub async fn get_all(
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let query = Self::joined_query(includes);

		let authorities = conn
			.interact(move |c| {
				query
					.select((
						PrimitiveAuthority::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.load(c)
			})
			.await??
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		Ok(authorities)
	}

	/// Delete an [`Authority`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(
		auth_id: i32,
		conn: &DbConn,
	) -> Result<(), Error> {
		conn.interact(move |conn| {
			use crate::db::authority::dsl::*;

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
	pub name:        String,
	pub description: Option<String>,
	pub created_by:  i32,
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
				use self::authority::dsl::*;

				diesel::insert_into(authority)
					.values(self)
					.returning(PrimitiveAuthority::as_returning())
					.get_result(conn)
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
	pub name:        Option<String>,
	pub description: Option<String>,
	pub updated_by:  i32,
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
			use crate::db::authority::dsl::*;

			diesel::update(authority.find(auth_id)).set(self).execute(conn)
		})
		.await??;

		let authority = Authority::get_by_id(auth_id, includes, conn).await?;

		info!("updated authority {authority:?}");

		Ok(authority)
	}
}
