use std::hash::Hash;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::Location;
use crate::schema::opening_time;

#[derive(
	Associations,
	Clone,
	Debug,
	Deserialize,
	Identifiable,
	Queryable,
	Selectable,
	Serialize,
)]
#[diesel(belongs_to(Location))]
#[diesel(table_name = crate::schema::opening_time)]
#[diesel(check_for_backend(Pg))]
#[serde(rename_all = "camelCase")]
pub struct OpeningTime {
	pub id:              i32,
	pub location_id:     i32,
	pub day:             NaiveDate,
	pub start_time:      NaiveTime,
	pub end_time:        NaiveTime,
	pub seat_count:      Option<i32>,
	pub reservable_from: Option<NaiveDateTime>,
	pub created_at:      NaiveDateTime,
	pub created_by:      Option<i32>,
	pub updated_at:      NaiveDateTime,
	pub updated_by:      Option<i32>,
}

impl Hash for OpeningTime {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state) }
}

impl PartialEq for OpeningTime {
	fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for OpeningTime {}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = crate::schema::opening_time)]
pub struct NewOpeningTime {
	pub location_id:     i32,
	pub day:             NaiveDate,
	pub start_time:      NaiveTime,
	pub end_time:        NaiveTime,
	pub seat_count:      Option<i32>,
	pub reservable_from: Option<NaiveDateTime>,
}

impl NewOpeningTime {
	/// Insert this [`NewOpeningTime`] into the database.
	///
	/// # Errors
	pub async fn insert(self, conn: &DbConn) -> Result<OpeningTime, Error> {
		let opening_time = conn
			.interact(|conn| {
				use self::opening_time::dsl::*;

				diesel::insert_into(opening_time)
					.values(self)
					.returning(OpeningTime::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(opening_time)
	}
}
