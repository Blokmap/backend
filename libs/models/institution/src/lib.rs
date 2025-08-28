#[macro_use]
extern crate tracing;

use ::translation::NewTranslation;
use base::{PaginatedData, PaginationConfig, manual_pagination};
use common::{DbConn, Error};
use db::{
	InstitutionCategory,
	creator,
	institution,
	profile,
	translation,
	updater,
};
use diesel::prelude::*;
use diesel::sql_types::Bool;
use primitives::{
	PrimitiveInstitution,
	PrimitiveProfile,
	PrimitiveTranslation,
};
use serde::{Deserialize, Serialize};

mod member;

pub use member::*;

pub type JoinedInstitutionData = (
	PrimitiveInstitution,
	PrimitiveTranslation,
	Option<PrimitiveProfile>,
	Option<PrimitiveProfile>,
);

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionIncludes {
	#[serde(default)]
	pub created_by: bool,
	#[serde(default)]
	pub updated_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct Institution {
	pub institution: PrimitiveInstitution,
	pub name:        PrimitiveTranslation,
	pub created_by:  Option<PrimitiveProfile>,
	pub updated_by:  Option<Option<PrimitiveProfile>>,
}

mod auto_type_helpers {
	pub use diesel::dsl::{LeftJoin as LeftOuterJoin, *};
}

impl Institution {
	/// Build a query with all required (dynamic) joins to select a full
	/// institution data tuple
	#[diesel::dsl::auto_type(no_type_alias, dsl_path = "auto_type_helpers")]
	fn joined_query(includes: InstitutionIncludes) -> _ {
		let inc_created: bool = includes.created_by;
		let inc_updated: bool = includes.updated_by;

		institution::table
			.inner_join(
				translation::table
					.on(institution::name_translation_id.eq(translation::id)),
			)
			.left_outer_join(
				creator.on(inc_created.into_sql::<Bool>().and(
					institution::created_by
						.eq(creator.field(profile::id).nullable()),
				)),
			)
			.left_outer_join(
				updater.on(inc_updated.into_sql::<Bool>().and(
					institution::updated_by
						.eq(updater.field(profile::id).nullable()),
				)),
			)
	}

	/// Construct a full [`Institution`] struct from the data returned by a
	/// joined query
	fn from_joined(
		includes: InstitutionIncludes,
		data: JoinedInstitutionData,
	) -> Self {
		Self {
			institution: data.0,
			name:        data.1,
			created_by:  if includes.created_by { data.2 } else { None },
			updated_by:  if includes.updated_by { Some(data.3) } else { None },
		}
	}

	#[instrument(skip(conn))]
	pub async fn get_all(
		includes: InstitutionIncludes,
		p_cfg: PaginationConfig,
		conn: &DbConn,
	) -> Result<PaginatedData<Vec<Self>>, Error> {
		let query = Self::joined_query(includes);

		let institutions = conn
			.interact(move |conn| {
				query
					.select((
						PrimitiveInstitution::as_select(),
						PrimitiveTranslation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|data| Self::from_joined(includes, data))
			.collect();

		manual_pagination(institutions, p_cfg)
	}

	/// Get an [`Institution`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		i_id: i32,
		includes: InstitutionIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query(includes);

		let institution = conn
			.interact(move |conn| {
				use self::institution::dsl::*;

				query
					.filter(id.eq(i_id))
					.select((
						PrimitiveInstitution::as_select(),
						PrimitiveTranslation::as_select(),
						creator.fields(profile::all_columns).nullable(),
						updater.fields(profile::all_columns).nullable(),
					))
					.get_result(conn)
			})
			.await??;

		let institution = Self::from_joined(includes, institution);

		Ok(institution)
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewInstitution {
	pub name_translation: NewTranslation,
	pub email:            Option<String>,
	pub phone_number:     Option<String>,
	pub street:           Option<String>,
	pub number:           Option<String>,
	pub zip:              Option<String>,
	pub city:             Option<String>,
	pub province:         Option<String>,
	pub country:          Option<String>,
	pub created_by:       i32,
	pub category:         InstitutionCategory,
	pub slug:             String,
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = institution)]
#[diesel(check_for_backend(Pg))]
pub struct InsertableNewInstitution {
	pub name_translation_id: i32,
	pub email:               Option<String>,
	pub phone_number:        Option<String>,
	pub street:              Option<String>,
	pub number:              Option<String>,
	pub zip:                 Option<String>,
	pub city:                Option<String>,
	pub province:            Option<String>,
	pub country:             Option<String>,
	pub created_by:          i32,
	pub category:            InstitutionCategory,
	pub slug:                String,
}

impl NewInstitution {
	#[instrument(skip(conn))]
	pub async fn insert(
		self,
		includes: InstitutionIncludes,
		conn: &DbConn,
	) -> Result<Institution, Error> {
		let institution = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use self::institution::dsl::institution;
					use self::translation::dsl::translation;

					let name = diesel::insert_into(translation)
						.values(self.name_translation)
						.returning(PrimitiveTranslation::as_returning())
						.get_result(conn)?;

					let new_institution = InsertableNewInstitution {
						name_translation_id: name.id,
						email:               self.email,
						phone_number:        self.phone_number,
						street:              self.street,
						number:              self.number,
						zip:                 self.zip,
						city:                self.city,
						province:            self.province,
						country:             self.country,
						created_by:          self.created_by,
						category:            self.category,
						slug:                self.slug,
					};

					let inst = diesel::insert_into(institution)
						.values(new_institution)
						.returning(PrimitiveInstitution::as_returning())
						.get_result(conn)?;

					Ok(inst)
				})
			})
			.await??;

		let institution =
			Institution::get_by_id(institution.id, includes, conn).await?;

		info!("inserted new institution {institution:?}");

		Ok(institution)
	}
}
