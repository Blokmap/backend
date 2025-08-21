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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderedImage {
	pub image: Image,
	pub index: i32,
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
	) -> Result<Vec<OrderedImage>, Error> {
		let imgs = conn
			.interact(move |conn| {
				use crate::db::image::dsl::*;
				use crate::db::location_image::dsl::*;

				location_image
					.filter(location_id.eq(l_id))
					.inner_join(image.on(image_id.eq(id)))
					.order(index.asc())
					.select((Self::as_select(), index))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(image, index)| OrderedImage { image, index })
			.collect();

		Ok(imgs)
	}

	/// Get all [`Image`]s for the locations with the given ids
	#[instrument(skip(l_ids, conn))]
	pub async fn get_for_locations(
		l_ids: Vec<i32>,
		conn: &DbConn,
	) -> Result<Vec<(i32, OrderedImage)>, Error> {
		let imgs = conn
			.interact(move |conn| {
				use crate::db::image::dsl::*;
				use crate::db::location;
				use crate::db::location_image::dsl::*;

				location::table
					.filter(location::id.eq_any(l_ids))
					.inner_join(location_image.on(location_id.eq(location::id)))
					.inner_join(image.on(image_id.eq(id)))
					.select((location::id, Self::as_select(), index))
					.get_results(conn)
			})
			.await??
			.into_iter()
			.map(|(id, image, index)| (id, OrderedImage { image, index }))
			.collect();

		Ok(imgs)
	}

	/// Reorder the images for the [`Location`](crate::Location) with the given
	/// id
	///
	/// # Warning
	/// This overwrites the entire list of `location_image`s for the location,
	/// and so may hide/delete images if the input doesn't refer to all images
	#[instrument(skip(conn))]
	pub async fn reorder(
		l_id: i32,
		new_order: Vec<NewLocationImage>,
		conn: &DbConn,
	) -> Result<Vec<OrderedImage>, Error> {
		// TODO: reordered images should be approved

		let images = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use crate::db::image::dsl::*;
					use crate::db::location_image::dsl::*;

					diesel::delete(location_image.filter(location_id.eq(l_id)))
						.execute(conn)?;

					diesel::insert_into(location_image)
						.values(new_order)
						.execute(conn)?;

					location_image
						.filter(location_id.eq(l_id))
						.inner_join(image.on(image_id.eq(id)))
						.order(index.asc())
						.select((Self::as_select(), index))
						.get_results(conn)
						.map_err(Into::into)
				})
			})
			.await??
			.into_iter()
			.map(|(image, index)| OrderedImage { image, index })
			.collect();

		Ok(images)
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
	/// Insert this [`NewImage`]
	#[instrument(skip(conn))]
	pub async fn insert(self, conn: &DbConn) -> Result<Image, Error> {
		let image = conn
			.interact(move |conn| {
				use crate::db::image::dsl::*;

				diesel::insert_into(image)
					.values(self)
					.returning(Image::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(image)
	}

	/// Insert this list of [`NewImage`]s into the database.
	#[instrument(skip(conn))]
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
	pub index:       i32,
}

#[derive(Clone, Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name = location_image)]
pub struct NewLocationImage {
	pub location_id: i32,
	pub image_id:    i32,
	pub index:       i32,
}

impl NewLocationImage {
	/// Insert this [`NewLocationImage`]
	#[instrument(skip(conn))]
	pub async fn insert(self, conn: &DbConn) -> Result<LocationImage, Error> {
		let loc_image = conn
			.interact(move |conn| {
				use crate::db::location_image::dsl::*;

				diesel::insert_into(location_image)
					.values(self)
					.returning(LocationImage::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(loc_image)
	}

	/// Insert this list of [`NewLocationImage`]s into the database.
	#[instrument(skip(conn))]
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
