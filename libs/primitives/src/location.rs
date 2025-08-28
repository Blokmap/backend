use chrono::NaiveDateTime;
use db::location;
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = location)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveLocation {
	pub id:                     i32,
	pub name:                   String,
	pub authority_id:           Option<i32>,
	pub description_id:         i32,
	pub excerpt_id:             i32,
	pub seat_count:             i32,
	pub is_reservable:          bool,
	pub max_reservation_length: Option<i32>,
	pub is_visible:             bool,
	pub street:                 String,
	pub number:                 String,
	pub zip:                    String,
	pub city:                   String,
	pub province:               String,
	pub country:                String,
	pub latitude:               f64,
	pub longitude:              f64,
	pub approved_at:            Option<NaiveDateTime>,
	pub approved_by:            Option<i32>,
	pub rejected_at:            Option<NaiveDateTime>,
	pub rejected_by:            Option<i32>,
	pub rejected_reason:        Option<String>,
	pub created_at:             NaiveDateTime,
	pub created_by:             Option<i32>,
	pub updated_at:             NaiveDateTime,
	pub updated_by:             Option<i32>,
}
