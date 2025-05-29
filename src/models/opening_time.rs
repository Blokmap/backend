use std::hash::Hash;

use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Jsonb;
use serde::{Deserialize, Serialize};

use super::Location;
use crate::schema::opening_time;
use crate::{DbConn, Error};

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
	pub id:            i32,
	pub location_id:   i32,
	pub start_time:    NaiveDateTime,
	pub end_time:      NaiveDateTime,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
	pub created_at:    NaiveDateTime,
	pub updated_at:    NaiveDateTime,
}

impl Hash for OpeningTime {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state) }
}

impl PartialEq for OpeningTime {
	fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for OpeningTime {}

impl<DB> Queryable<Jsonb, DB> for OpeningTime
where
	DB: Backend,
	OpeningTime: FromSql<Jsonb, DB>,
{
	type Row = OpeningTime;

	fn build(row: Self::Row) -> diesel::deserialize::Result<Self> { Ok(row) }
}

impl<DB> FromSql<Jsonb, DB> for OpeningTime
where
	DB: Backend,
	serde_json::Value: FromSql<Jsonb, DB>,
{
	fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
		let value = <serde_json::Value as FromSql<Jsonb, DB>>::from_sql(bytes)?;
		Ok(serde_json::from_value(value)?)
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = crate::schema::opening_time)]
pub struct NewOpeningTime {
	pub location_id:   i32,
	pub start_time:    NaiveDateTime,
	pub end_time:      NaiveDateTime,
	pub seat_count:    Option<i32>,
	pub is_reservable: Option<bool>,
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
