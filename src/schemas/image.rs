use common::Error;
use models::{Image, OrderedImage};
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::schemas::BuildResponse;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageResponse {
	pub id:    i32,
	pub url:   String,
	pub index: Option<i32>,
}

impl BuildResponse<ImageResponse> for Image {
	fn build_response(self, config: &Config) -> Result<ImageResponse, Error> {
		let url = if let Some(file_path) = &self.file_path {
			let url = config.static_url.join(file_path)?;
			Ok(url)
		} else if let Some(image_url) = &self.image_url {
			let url = image_url.parse()?;
			Ok(url)
		} else {
			Err(Error::Infallible("no valid image url".to_string()))
		};

		let url = url?;

		let response = ImageResponse {
			id:    self.id,
			url:   url.to_string(),
			index: None,
		};

		Ok(response)
	}
}

impl BuildResponse<ImageResponse> for OrderedImage {
	fn build_response(
		self,
		config: &Config,
	) -> Result<ImageResponse, common::Error> {
		let mut response = self.image.build_response(config)?;
		response.index = Some(self.index);

		Ok(response)
	}
}
