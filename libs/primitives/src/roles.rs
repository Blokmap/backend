use chrono::NaiveDateTime;
use db::{authority_role, institution_role, location_role};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = institution_role)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveInstitutionRole {
	pub id:             i32,
	pub institution_id: i32,
	pub name:           String,
	pub permissions:    i64,
	pub created_at:     NaiveDateTime,
	pub created_by:     Option<i32>,
	pub updated_at:     NaiveDateTime,
	pub updated_by:     Option<i32>,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = authority_role)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveAuthorityRole {
	pub id:           i32,
	pub authority_id: i32,
	pub name:         String,
	pub permissions:  i64,
	pub created_at:   NaiveDateTime,
	pub created_by:   Option<i32>,
	pub updated_at:   NaiveDateTime,
	pub updated_by:   Option<i32>,
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = location_role)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveLocationRole {
	pub id:          i32,
	pub location_id: i32,
	pub name:        String,
	pub permissions: i64,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<i32>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<i32>,
}
