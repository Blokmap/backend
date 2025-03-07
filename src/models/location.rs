use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::DbConn;
use crate::error::Error;
use crate::schema::location;

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = crate::schema::location)]
#[serde(rename_all = "camelCase")]
pub struct Location {
	pub id:              i32,
	pub name:            String,
	pub description_key: Uuid,
	pub excerpt_key:     Uuid,
	pub seat_count:      i32,
	pub is_reservable:   bool,
	pub is_visible:      bool,
	pub street:          String,
	pub number:          String,
	pub zip:             String,
	pub city:            String,
	pub province:        String,
	pub latitude:        f64,
	pub longitude:       f64,
	pub cell_idx:        i32,
	pub created_at:      DateTime<Utc>,
	pub updated_at:      DateTime<Utc>,
}

#[derive(Queryable, Identifiable, Associations, Serialize, Debug)]
#[diesel(belongs_to(Location))]
#[diesel(table_name = crate::schema::opening_time)]
#[serde(rename_all = "camelCase")]
pub struct OpeningTime {
	pub id:            i32,
	pub location_id:   i32,
	pub start_time:    DateTime<Utc>,
	pub end_time:      DateTime<Utc>,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
	pub created_at:    DateTime<Utc>,
	pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::location)]
pub struct NewLocation {
	pub name:            String,
	pub description_key: Uuid,
	pub excerpt_key:     Uuid,
	pub seat_count:      i32,
	pub is_reservable:   bool,
	pub is_visible:      bool,
	pub street:          String,
	pub number:          String,
	pub zip:             String,
	pub city:            String,
	pub province:        String,
	pub latitude:        f64,
	pub longitude:       f64,
	pub cell_idx:        i32,
}

impl NewLocation {
	/// Insert this [`NewLocation`]
	pub(crate) async fn insert(self, conn: &DbConn) -> Result<Location, Error> {
		let location = conn
			.interact(|conn| {
				use self::location::dsl::*;

				diesel::insert_into(location)
					.values(self)
					.returning(Location::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(location)
	}
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::opening_time)]
pub struct NewOpeningTime {
	pub location_id:   i32,
	pub start_time:    DateTime<Utc>,
	pub end_time:      DateTime<Utc>,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
}
