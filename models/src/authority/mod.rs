use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};

use crate::SimpleProfile;
use crate::schema::{authority, creator, simple_profile, updater};

mod member;

pub use member::*;

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
	pub updated_by: Option<Option<SimpleProfile>>,
	pub created_by: Option<Option<SimpleProfile>>,
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
	pub updated_at:  NaiveDateTime,
}

impl Authority {
	/// Get a single [`Authority`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		auth_id: i32,
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let authority: (
			PrimitiveAuthority,
			Option<SimpleProfile>,
			Option<SimpleProfile>,
		) = conn
			.interact(move |conn| {
				use crate::schema::authority::dsl::*;

				authority
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							created_by.eq(
								creator.field(simple_profile::id).nullable(),
							),
						),
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							updated_by.eq(
								updater.field(simple_profile::id).nullable(),
							),
						),
					))
					.filter(id.eq(auth_id))
					.select((
						PrimitiveAuthority::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let authority = Self {
			authority:  authority.0,
			created_by: if includes.created_by {
				Some(authority.1)
			} else {
				None
			},
			updated_by: if includes.updated_by {
				Some(authority.2)
			} else {
				None
			},
		};

		Ok(authority)
	}

	/// Get all [`Authorities`]s from the database, optionally including related
	/// profiles.
	#[instrument(skip(conn))]
	pub async fn get_all(
		includes: AuthorityIncludes,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let authorities = conn
			.interact(move |c| {
				use crate::schema::authority::dsl::*;

				authority
					.left_outer_join(creator.on(
						includes.created_by.into_sql::<Bool>().and(
							created_by.eq(
								creator.field(simple_profile::id).nullable(),
							),
						),
					))
					.left_outer_join(updater.on(
						includes.updated_by.into_sql::<Bool>().and(
							updated_by.eq(
								updater.field(simple_profile::id).nullable(),
							),
						),
					))
					.select((
						PrimitiveAuthority::as_select(),
						creator.fields(simple_profile::all_columns).nullable(),
						updater.fields(simple_profile::all_columns).nullable(),
					))
					.load(c)
			})
			.await??
			.into_iter()
			.map(|(authority, cr, up)| {
				Authority {
					authority,
					created_by: if includes.created_by {
						Some(cr)
					} else {
						None
					},
					updated_by: if includes.updated_by {
						Some(up)
					} else {
						None
					},
				}
			})
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
			use crate::schema::authority::dsl::*;

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
			use crate::schema::authority::dsl::*;

			diesel::update(authority.find(auth_id)).set(self).execute(conn)
		})
		.await??;

		let authority = Authority::get_by_id(auth_id, includes, conn).await?;

		info!("updated authority {authority:?}");

		Ok(authority)
	}
}
