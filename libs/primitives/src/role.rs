use chrono::NaiveDateTime;
use db::role;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = role)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveRole {
	pub id:          i32,
	pub name:        String,
	pub colour:      String,
	pub permissions: i64,
	pub created_at:  NaiveDateTime,
	pub created_by:  Option<i32>,
	pub updated_at:  NaiveDateTime,
	pub updated_by:  Option<i32>,
}
