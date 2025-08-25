use chrono::NaiveDateTime;
use db::image;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = image)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveImage {
	pub id:          i32,
	pub file_path:   Option<String>,
	pub uploaded_at: NaiveDateTime,
	pub uploaded_by: Option<i32>,
	pub image_url:   Option<String>,
}
