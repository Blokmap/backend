use chrono::NaiveDateTime;
use db::{InstitutionCategory, institution};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = institution)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveInstitution {
	pub id:                  i32,
	pub name_translation_id: i32,
	pub email:               Option<String>,
	pub phone_number:        Option<String>,
	pub street:              Option<String>,
	pub number:              Option<String>,
	pub zip:                 Option<String>,
	pub city:                Option<String>,
	pub province:            Option<String>,
	pub country:             Option<String>,
	pub created_at:          NaiveDateTime,
	pub created_by:          i32,
	pub updated_at:          NaiveDateTime,
	pub updated_by:          Option<i32>,
	pub category:            InstitutionCategory,
	pub slug:                String,
}
