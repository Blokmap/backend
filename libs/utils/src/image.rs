use std::io::Cursor;
use std::path::PathBuf;

use axum::body::Bytes;
use common::Error;
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};
use image::{ColorType, ImageReader};
use uuid::Uuid;

/// Resize an image to 1024x1024 (as close as possible while preserving aspect
/// ratio)
///
/// # Panics
/// Panics if the decoder can't infer the images pixel type
#[inline]
pub fn resize_image(
	bytes: Bytes,
) -> Result<(Image<'static>, u32, u32, ColorType), Error> {
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

	Ok((dst_image, dst_width, dst_height, src_image.color()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageOwner {
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
pub fn generate_image_filepaths(
	id: i32,
	owner: ImageOwner,
) -> Result<(PathBuf, PathBuf), Error> {
	let owner_chunk = owner.as_url_chunk();

	let image_uuid = Uuid::new_v4().to_string();
	let rel_filepath = PathBuf::from(owner_chunk)
		.join(id.to_string())
		.join(image_uuid)
		.with_extension("webp");

	let abs_filepath = PathBuf::from("/mnt/files").join(&rel_filepath);

	// Ensure all parent directories exist
	let prefix = abs_filepath.parent().unwrap();
	std::fs::create_dir_all(prefix)?;

	Ok((abs_filepath, rel_filepath))
}
