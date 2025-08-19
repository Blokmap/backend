use chrono::NaiveDateTime;
use common::{DbConn, Error};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::{Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::db::{image, location_image};

#[derive(
	Clone, Debug, Deserialize, Identifiable, Queryable, Selectable, Serialize,
)]
#[diesel(table_name = image)]
#[diesel(check_for_backend(Pg))]
pub struct Image {
	pub id:          i32,
	pub file_path:   Option<String>,
	pub uploaded_at: NaiveDateTime,
	pub uploaded_by: i32,
	pub image_url:   Option<String>,
}

impl Image {
	pub async fn get_by_id(img_id: i32, conn: &DbConn) -> Result<Self, Error> {
		let img = conn
			.interact(move |conn| {
				use self::image::dsl::*;

				image.find(img_id).first(conn)
			})
			.await??;

		Ok(img)
	}

	pub async fn delete_by_id(img_id: i32, conn: &DbConn) -> Result<(), Error> {
		conn.interact(move |conn| {
			use self::image::dsl::*;

			diesel::delete(image.find(img_id)).execute(conn)
		})
		.await??;

		Ok(())
	}

	/// Get all [`Image`]s for a location with the given id
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		l_id: i32,
		conn: &DbConn,
	) -> Result<Vec<Self>, Error> {
		let imgs = conn
			.interact(move |conn| {
				use crate::db::image::dsl::*;
				use crate::db::location;
				use crate::db::location_image::dsl::*;

				location::table
					.find(l_id)
					.inner_join(location_image.on(location_id.eq(location::id)))
					.inner_join(image.on(image_id.eq(id)))
					.select(Self::as_select())
					.get_results(conn)
			})
			.await??;

		Ok(imgs)
	}

	/// Get all [`Image`]s for the locations with the given ids
	#[instrument(skip(l_ids, conn))]
	pub async fn get_for_locations(
		l_ids: Vec<i32>,
		conn: &DbConn,
	) -> Result<Vec<(i32, Self)>, Error> {
		let imgs = conn
			.interact(move |conn| {
				use crate::db::image::dsl::*;
				use crate::db::location;
				use crate::db::location_image::dsl::*;

				location::table
					.filter(location::id.eq_any(l_ids))
					.inner_join(location_image.on(location_id.eq(location::id)))
					.inner_join(image.on(image_id.eq(id)))
					.select((location::id, Self::as_select()))
					.get_results(conn)
			})
			.await??;

		Ok(imgs)
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = image)]
pub struct NewImage {
	pub file_path:   Option<String>,
	pub uploaded_by: i32,
	pub image_url:   Option<String>,
}

impl NewImage {
	/// Insert this list of [`NewImage`]s into the database.
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
