#[macro_use]
extern crate tracing;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
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

impl PrimitiveReservation {
	/// Get a [`PrimitiveReservation`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(r_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let reservation = conn
			.interact(move |conn| {
				use self::reservation::dsl::*;

				reservation
					.find(r_id)
					.select(Self::as_select())
					.get_result(conn)
			})
			.await??;

		Ok(reservation)
	}

	/// Count reservations by opening time id.
	#[instrument(skip(conn))]
	pub async fn count_by_opening_time_id(
		opening_time_id: i32,
		conn: &DbConn,
	) -> Result<i64, Error> {
		let count = conn
			.interact(move |conn| {
				use self::reservation::dsl::*;

				reservation
					.filter(opening_time_id.eq(opening_time_id))
					.count()
					.get_result(conn)
			})
			.await??;

		Ok(count)
	}
}
