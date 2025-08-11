use models::Image;
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::schemas::BuildResponse;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageResponse {
	url: String,
}

impl BuildResponse<ImageResponse> for Image {
	fn build_response(self, config: &Config) -> ImageResponse {
		ImageResponse {
			url: config.static_url.join(&self.file_path).unwrap().to_string(),
		}
	}
}
