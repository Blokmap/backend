#[macro_use]
extern crate tracing;

use chrono::NaiveDateTime;
use common::{DbConn, Error};
use db::{image, location, location_image, profile};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use diesel::{Identifiable, Queryable, Selectable};
use primitives::{PrimitiveImage, PrimitiveProfile};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct ImageIncludes {
	#[serde(default)]
	pub uploaded_by: bool,
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(Pg))]
pub struct Image {
	#[diesel(embed)]
	pub primitive:   PrimitiveImage,
	#[diesel(embed)]
	pub uploaded_by: Option<PrimitiveProfile>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrderedImage {
	pub image: Image,
	pub index: i32,
}

impl Image {
	/// Build a query with all required (dynamic) joins to select a full
	/// image data tuple
	#[diesel::dsl::auto_type(no_type_alias)]
	fn query(includes: ImageIncludes) -> _ {
		let inc_uploaded: bool = includes.uploaded_by;

		image::table.left_join(
			profile::table.on(inc_uploaded
				.into_sql::<Bool>()
				.and(image::uploaded_by.eq(profile::id.nullable()))),
		)
	}

	/// Get an [`Image`]s given its id
	#[instrument(skip(conn))]
	pub async fn get_by_id(
		i_id: i32,
		includes: ImageIncludes,
		conn: &DbConn,
	) -> Result<Self, Error> {
		let query = Self::query(includes);

		let img = conn
			.interact(move |conn| {
				query.select(Self::as_select()).get_result(conn)
			})
			.await??;

		Ok(img)
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
		includes: ImageIncludes,
		conn: &DbConn,
	) -> Result<Vec<OrderedImage>, Error> {
		let query = Self::query(includes);

		let imgs = conn
			.interact(move |conn| {
				use self::image::dsl::*;
				use self::location_image::dsl::*;

				location_image
					.filter(location_id.eq(l_id))
					.inner_join(query.on(image_id.eq(id)))
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
		includes: ImageIncludes,
		conn: &DbConn,
	) -> Result<Vec<(i32, OrderedImage)>, Error> {
		let query = Self::query(includes);

		let imgs = conn
			.interact(move |conn| {
				use self::image::dsl::*;
				use self::location;
				use self::location_image::dsl::*;

				location::table
					.filter(location::id.eq_any(l_ids))
					.inner_join(location_image.on(location_id.eq(location::id)))
					.inner_join(query.on(image_id.eq(id)))
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
		includes: ImageIncludes,
		conn: &DbConn,
	) -> Result<Vec<OrderedImage>, Error> {
		// TODO: reordered images should be approved

		let query = Self::query(includes);

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
						.inner_join(query.on(image_id.eq(id)))
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
	/// Insert a [`NewImage`] with an index for a specific [`Location`]
	#[instrument(skip(conn))]
	pub async fn insert_for_location(
		self,
		loc_id: i32,
		image_index: i32,
		conn: &DbConn,
	) -> Result<OrderedImage, Error> {
		let primitive = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use self::image::dsl::*;
					use self::location_image::dsl::*;

					let inserted_image = diesel::insert_into(image)
						.values(self)
						.returning(PrimitiveImage::as_returning())
						.get_result(conn)?;

					let new_location_image = NewLocationImage {
						location_id: loc_id,
						image_id:    inserted_image.id,
						index:       image_index,
					};

					diesel::insert_into(location_image)
						.values(new_location_image)
						.execute(conn)?;

					Ok(inserted_image)
				})
			})
			.await??;

		let image =
			Image::get_by_id(primitive.id, ImageIncludes::default(), conn)
				.await?;

		let ordered_image = OrderedImage { image, index: image_index };

		Ok(ordered_image)
	}

	/// Insert a [`NewImage`] for a specific [`Profile`]
	#[instrument(skip(conn))]
	pub async fn insert_for_profile(
		self,
		p_id: i32,
		conn: &DbConn,
	) -> Result<Image, Error> {
		let primitive = conn
			.interact(move |conn| {
				conn.transaction::<_, Error, _>(|conn| {
					use self::profile::dsl::*;

					let image_record = diesel::insert_into(image::table)
						.values(self)
						.returning(PrimitiveImage::as_returning())
						.get_result(conn)?;

					diesel::update(profile.find(p_id))
						.set(avatar_image_id.eq(image_record.id))
						.execute(conn)?;

					Ok(image_record)
				})
			})
			.await??;

		let image =
			Image::get_by_id(primitive.id, ImageIncludes::default(), conn)
				.await?;

		Ok(image)
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
