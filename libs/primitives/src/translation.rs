use chrono::NaiveDateTime;
use db::translation;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = translation)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveTranslation {
	pub id:         i32,
	pub nl:         Option<String>,
	pub en:         Option<String>,
	pub fr:         Option<String>,
	pub de:         Option<String>,
	pub created_at: NaiveDateTime,
	pub created_by: Option<i32>,
	pub updated_at: NaiveDateTime,
	pub updated_by: Option<i32>,
}
