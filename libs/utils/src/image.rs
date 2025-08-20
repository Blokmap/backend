use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::{Path, PathBuf};

use axum::body::Bytes;
use common::{DbConn, Error};
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::codecs::webp::WebPEncoder;
use image::{ColorType, ImageEncoder, ImageReader};
use models::{Location, NewImage, Profile};
use rayon::prelude::*;
use uuid::Uuid;

/// Store a list of images for the given location
pub async fn store_location_images(
	uploader_id: i32,
	location_id: i32,
	bytes: &[Bytes],
	conn: &DbConn,
) -> Result<Vec<models::Image>, Error> {
	let images = bytes
		.into_par_iter()
		.map(|bytes| {
			let (image, color_type) = resize_image(bytes)?;
			let (abs_filepath, rel_filepath) =
				generate_image_filepaths(ImageOwner::Location, location_id)?;

			save_image_file(&abs_filepath, &image, color_type)?;

			let new_image = NewImage {
				file_path:   Some(rel_filepath.to_string_lossy().into_owned()),
				uploaded_by: uploader_id,
				image_url:   None,
			};

			Ok(new_image)
		})
		.collect::<Result<Vec<NewImage>, Error>>()?;

	let images = Location::insert_images(location_id, images, conn).await?;

	Ok(images)
}

/// Store an image for the given profile
pub async fn store_profile_image(
	profile_id: i32,
	bytes: &Bytes,
	conn: &DbConn,
) -> Result<models::Image, Error> {
	let (image, color_type) = resize_image(bytes)?;
	let (abs_filepath, rel_filepath) =
		generate_image_filepaths(ImageOwner::Profile, profile_id)?;

	save_image_file(&abs_filepath, &image, color_type)?;

	let new_image = NewImage {
		file_path:   Some(rel_filepath.to_string_lossy().into_owned()),
		uploaded_by: profile_id,
		image_url:   None,
	};

	let image = Profile::insert_avatar(profile_id, new_image, conn).await?;

	Ok(image)
}

/// Delete an image from both the database and disk storage
pub async fn delete_image(id: i32, conn: &DbConn) -> Result<(), Error> {
	// Delete the image record before the file to prevent dangling
	let image = models::Image::get_by_id(id, conn).await?;
	models::Image::delete_by_id(id, conn).await?;

	if let Some(file_path) = &image.file_path {
		let filepath = PathBuf::from("/mnt/files").join(file_path);
		std::fs::remove_file(filepath)?;
	}

	Ok(())
}

/// Save an image to a file
fn save_image_file(
	path: &Path,
	image: &Image<'static>,
	color_type: ColorType,
) -> Result<(), Error> {
	let mut file = BufWriter::new(File::create(path)?);

	WebPEncoder::new_lossless(&mut file).write_image(
		image.buffer(),
		image.width(),
		image.height(),
		color_type.into(),
	)?;

	file.flush()?;

	Ok(())
}

/// Resize an image to 1024x1024 (as close as possible while preserving aspect
/// ratio)
///
/// # Panics
/// Panics if the decoder can't infer the images pixel type
#[inline]
fn resize_image(bytes: &Bytes) -> Result<(Image<'static>, ColorType), Error> {
	let image_reader =
		ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;

	let src_image = image_reader.decode()?;

	// Set width to 1024 but scale height to preserve aspect ratio
	#[allow(clippy::cast_precision_loss)]
	let src_ratio = src_image.height() as f32 / src_image.width() as f32;
	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::cast_sign_loss)]
	let dst_height = (1024.0 * src_ratio) as u32;
	let dst_width = 1024;

	let mut dst_image =
		Image::new(dst_width, dst_height, src_image.pixel_type().unwrap());

	let mut resizer = Resizer::new();
	resizer.resize(&src_image, &mut dst_image, None)?;

	Ok((dst_image, src_image.color()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageOwner {
	Profile,
	Location,
}

impl ImageOwner {
	fn as_url_chunk(self) -> &'static str {
		match self {
			Self::Profile => "profile",
			Self::Location => "location",
		}
	}
}

/// Generate both an absolute and relative filepath for a new image
///
/// The absolute path is used for writing to disk, the relative path is used
/// by the API
///
/// # Panics
/// Panics if some wandering cosmic ray decides to mess up the file path
/// generation
#[inline]
fn generate_image_filepaths(
	owner_type: ImageOwner,
	owner_id: i32,
) -> Result<(PathBuf, PathBuf), Error> {
	let owner_chunk = owner_type.as_url_chunk();

	let image_uuid = Uuid::new_v4().to_string();
	let rel_filepath = PathBuf::from(owner_chunk)
		.join(owner_id.to_string())
		.join(image_uuid)
		.with_extension("webp");

	let abs_filepath = PathBuf::from("/mnt/files").join(&rel_filepath);

	// Ensure all parent directories exist
	let prefix = abs_filepath.parent().unwrap();
	std::fs::create_dir_all(prefix)?;

	Ok((abs_filepath, rel_filepath))
}
