use chrono::NaiveDateTime;
use db::review;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = review)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveReview {
	pub id:          i32,
	pub profile_id:  i32,
	pub location_id: i32,
	pub rating:      i32,
	pub body:        Option<String>,
	pub created_at:  NaiveDateTime,
	pub updated_at:  NaiveDateTime,
}
