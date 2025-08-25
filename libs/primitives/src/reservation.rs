use chrono::NaiveDateTime;
use db::{ReservationState, reservation};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(Pg))]
pub struct PrimitiveReservation {
	pub id:               i32,
	pub profile_id:       i32,
	pub state:            ReservationState,
	pub opening_time_id:  i32,
	pub base_block_index: i32,
	pub block_count:      i32,
	pub created_at:       NaiveDateTime,
	pub updated_at:       NaiveDateTime,
	pub confirmed_at:     Option<NaiveDateTime>,
}
