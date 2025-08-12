use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::db::{
	creator,
	institution,
	institution_name,
	institution_slug,
	profile,
	translation,
	updater,
};
use crate::{PrimitiveProfile, PrimitiveTranslation};

pub type JoinedInstitutionData = (
	PrimitiveInstitution,
	PrimitiveTranslation,
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
	pub slug:        PrimitiveTranslation,
	pub created_by:  Option<Option<PrimitiveProfile>>,
	pub updated_by:  Option<Option<PrimitiveProfile>>,
}

#[derive(
	Clone, Copy, DbEnum, Debug, Default, Deserialize, PartialEq, Eq, Serialize,
)]
#[ExistingTypePath = "crate::db::sql_types::InstitutionCategory"]
pub enum InstitutionCategory {
	#[default]
	Education,
	Organisation,
	Government,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = institution)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveInstitution {
	id:                  i32,
	name_translation_id: i32,
	slug_translation_id: i32,
	email:               Option<String>,
	phone_number:        Option<String>,
	street:              Option<String>,
	number:              Option<String>,
	zip:                 Option<String>,
	city:                Option<String>,
	province:            Option<String>,
	country:             Option<String>,
	created_at:          NaiveDateTime,
	created_by:          Option<i32>,
	updated_at:          NaiveDateTime,
	updated_by:          Option<i32>,
	category:            InstitutionCategory,
	slug:                String,
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
				institution_name.on(institution::name_translation_id
					.eq(institution_name.field(translation::id))),
			)
			.inner_join(
				institution_slug.on(institution::slug_translation_id
					.eq(institution_slug.field(translation::id))),
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
			slug:        data.2,
			created_by:  if includes.created_by { Some(data.3) } else { None },
			updated_by:  if includes.updated_by { Some(data.4) } else { None },
		}
	}

	/// Get a [`Reservation`] given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		i_id: i32,
		includes: InstitutionIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::joined_query(includes);

		let institution = conn
			.interact(move |conn| {
				use crate::db::institution::dsl::*;

				query
					.filter(id.eq(i_id))
					.select((
						PrimitiveInstitution::as_select(),
						institution_name.fields(translation::all_columns),
						institution_slug.fields(translation::all_columns),
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
