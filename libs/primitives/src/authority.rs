use chrono::NaiveDateTime;
use db::authority;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	AsChangeset,
	Clone,
	Debug,
	Deserialize,
	Identifiable,
	Queryable,
	Selectable,
	Serialize,
)]
#[diesel(table_name = authority)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveAuthority {
	pub id:             i32,
	pub name:           String,
	pub description:    Option<String>,
	pub institution_id: Option<i32>,
	pub created_at:     NaiveDateTime,
	pub created_by:     Option<i32>,
	pub updated_at:     NaiveDateTime,
	pub updated_by:     Option<i32>,
}
