use models::Image;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageResponse {
	id:        i32,
	file_path: String,
}

impl From<Image> for ImageResponse {
	fn from(value: Image) -> Self {
		Self { id: value.id, file_path: value.file_path }
	}
}
