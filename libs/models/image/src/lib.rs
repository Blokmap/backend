#[macro_use]
extern crate tracing;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
use db::{image, location, location_image, profile};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel::{Identifiable, Queryable, Selectable};
use models_common::JoinParts;
use primitive_image::PrimitiveImage;
use primitive_profile::PrimitiveProfile;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct ImageIncludes {
	#[serde(default)]
	pub uploaded_by: bool,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = image)]
#[diesel(check_for_backend(Pg))]
pub struct ImageParts {
	#[diesel(embed)]
	pub primitive:   PrimitiveImage,
	#[diesel(embed)]
	pub uploaded_by: Option<PrimitiveProfile>,
}

impl JoinParts for ImageParts {
	type Includes = ImageIncludes;
	type Target = Image;

	fn join(self, includes: Self::Includes) -> Self::Target {
		Image {
			primitive:   self.primitive,
			uploaded_by: if includes.uploaded_by {
				Some(self.uploaded_by)
			} else {
				None
			},
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Image {
	pub primitive:   PrimitiveImage,
	pub uploaded_by: Option<Option<PrimitiveProfile>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderedImage {
	pub image: PrimitiveImage,
	pub index: i32,
}

impl Image {
	/// Build a query with all required (dynamic) joins to select a full
	/// image data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn _query(includes: ImageIncludes) -> _ {
		let inc_uploaded: bool = includes.uploaded_by;

		image::table.left_join(
			profile::table.on(inc_uploaded
				.into_sql::<Bool>()
				.and(image::uploaded_by.eq(profile::id.nullable()))),
		)
	}

	/// Delete an [`Image`] given its id
	#[instrument(skip(conn))]
	pub async fn delete_by_id(
		img_id: i32,
		conn: &DbConn,
	) -> Result<PrimitiveImage, Error> {
		let image = conn
			.interact(move |conn| {
				use self::image::dsl::*;

				diesel::delete(image.find(img_id))
					.returning(PrimitiveImage::as_returning())
					.get_result(conn)
			})
			.await??;

		Ok(image)
	}

	/// Get all [`Image`]s for a location with the given id
	#[instrument(skip(conn))]
	pub async fn get_for_location(
		l_id: i32,
		conn: &DbConn,
	) -> Result<Vec<OrderedImage>, Error> {
		let imgs = conn
			.interact(move |conn| {
				use self::image::dsl::*;
				use self::location_image::dsl::*;

				location_image
					.filter(location_id.eq(l_id))
					.inner_join(image.on(image_id.eq(id)))
					.order(index.asc())
					.select((PrimitiveImage::as_select(), index))
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
				use self::image::dsl::*;
				use self::location;
				use self::location_image::dsl::*;

				location::table
					.filter(location::id.eq_any(l_ids))
					.inner_join(location_image.on(location_id.eq(location::id)))
					.inner_join(image.on(image_id.eq(id)))
					.select((location::id, PrimitiveImage::as_select(), index))
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
					use self::image::dsl::*;
					use self::location_image::dsl::*;

					diesel::delete(location_image.filter(location_id.eq(l_id)))
						.execute(conn)?;

					diesel::insert_into(location_image)
						.values(new_order)
						.execute(conn)?;

					location_image
						.filter(location_id.eq(l_id))
						.inner_join(image.on(image_id.eq(id)))
						.order(index.asc())
						.select((PrimitiveImage::as_select(), index))
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
	pub async fn insert(self, conn: &DbConn) -> Result<PrimitiveImage, Error> {
		let image = conn
			.interact(move |conn| {
				use self::image::dsl::*;

				diesel::insert_into(image)
					.values(self)
					.returning(PrimitiveImage::as_returning())
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
	) -> Result<Vec<PrimitiveImage>, Error> {
		let images = conn
			.interact(move |conn| {
				use self::image::dsl::*;

				diesel::insert_into(image)
					.values(v)
					.returning(PrimitiveImage::as_returning())
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
				use self::location_image::dsl::*;

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
