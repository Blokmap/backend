use common::Error;
use models::Image;
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::schemas::BuildResponse;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageResponse {
	url: String,
}

impl BuildResponse<ImageResponse> for Image {
	fn build_response(self, config: &Config) -> Result<ImageResponse, Error> {
		let url = if let Some(file_path) = &self.file_path {
			let url = config.frontend_url.join(file_path)?;
			Ok(url)
		} else if let Some(image_url) = &self.image_url {
			let url = image_url.parse()?;
			Ok(url)
		} else {
			Err(Error::Infallible("no valid image url".to_string()))
		};

		let url = url?;

		Ok(ImageResponse { url: url.to_string() })
	}
}
