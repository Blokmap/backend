#[macro_use]
extern crate tracing;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
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
	pub rejected_at:            Option<NaiveDateTime>,
	pub rejected_reason:        Option<String>,
	pub created_at:             NaiveDateTime,
	pub updated_at:             NaiveDateTime,
}

impl PrimitiveLocation {
	/// Get a [`PrimitiveLocation`] by its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(l_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let location = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				location.find(l_id).select(Self::as_select()).first(conn)
			})
			.await??;

		Ok(location)
	}

	/// Get a list of [`PrimitiveLocation`]s given a list of ids
	#[instrument(skip(conn))]
	pub async fn get_by_ids(
		l_ids: Vec<i32>,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let location = conn
			.interact(move |conn| {
				use self::location::dsl::*;

				location
					.filter(id.eq_any(l_ids))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(location)
	}
}
