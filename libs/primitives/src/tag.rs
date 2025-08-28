use chrono::NaiveDateTime;
use db::tag;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = tag)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveTag {
	pub id:                  i32,
	pub name_translation_id: i32,
	pub created_at:          NaiveDateTime,
	pub created_by:          Option<i32>,
	pub updated_at:          NaiveDateTime,
	pub updated_by:          Option<i32>,
}
