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
	pub id:         i32,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}
