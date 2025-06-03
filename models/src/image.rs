use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::{Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::{image, location_image};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = image)]
#[diesel(check_for_backend(Pg))]
pub struct Image {
	pub id:          i32,
	pub file_path:   String,
	pub uploaded_at: NaiveDateTime,
	pub uploaded_by: i32,
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = image)]
pub struct NewImage {
	pub file_path:   String,
	pub uploaded_by: i32,
}

impl NewImage {
	/// Insert this list of [`NewImage`]s into the database.
	///
	/// # Errors
	pub async fn bulk_insert(
		v: Vec<Self>,
		conn: &DbConn,
	) -> Result<Vec<Image>, Error> {
		let images = conn
			.interact(move |conn| {
				use self::image::dsl::*;

				diesel::insert_into(image)
					.values(v)
					.returning(Image::as_returning())
					.get_results(conn)
			})
			.await??;

		Ok(images)
	}
}

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = location_image)]
#[diesel(primary_key(location_id, image_id))]
#[diesel(check_for_backend(Pg))]
pub struct LocationImage {
	pub location_id: i32,
	pub image_id:    i32,
	pub approved_at: Option<NaiveDateTime>,
	pub approved_by: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = location_image)]
pub struct NewLocationImage {
	pub location_id: i32,
	pub image_id:    i32,
}

impl NewLocationImage {
	/// Insert this list of [`NewLocationImage`]s into the database.
	///
	/// # Errors
	pub async fn bulk_insert(
		v: Vec<Self>,
		conn: &DbConn,
	) -> Result<Vec<LocationImage>, Error> {
		let images = conn
			.interact(move |conn| {
				use self::location_image::dsl::*;

				diesel::insert_into(location_image)
					.values(v)
					.returning(LocationImage::as_returning())
					.get_results(conn)
			})
			.await??;

		Ok(images)
	}
}
