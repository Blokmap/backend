#[macro_use]
extern crate tracing;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
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

impl PrimitiveImage {
	#[instrument(skip(conn))]
	pub async fn get_by_id(img_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let img = conn
			.interact(move |conn| {
				use self::image::dsl::*;

				image.find(img_id).first(conn)
			})
			.await??;

		Ok(img)
	}

	#[instrument(skip(conn))]
	pub async fn delete_by_id(img_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::image::dsl::*;

			diesel::delete(image.find(img_id)).execute(conn)
		})
		.await??;

		Ok(())
	}
}
