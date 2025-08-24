use axum::body::Bytes;
use axum::extract::Multipart;
use axum::extract::multipart::Field;
use common::{Error, MultipartParseError};
use models::OrderedImage;
use primitive_image::PrimitiveImage;
use serde::{Deserialize, Serialize};
use utils::image::{ImageVariant, OrderedImageVariant};

use crate::Config;
use crate::schemas::BuildResponse;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImageResponse {
	pub id:    i32,
	pub url:   String,
	pub index: Option<i32>,
}

impl BuildResponse<ImageResponse> for PrimitiveImage {
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

#[derive(Clone, Debug)]
pub enum CreateImageRequest {
	Image(Bytes),
	Url(String),
}

impl CreateImageRequest {
	pub async fn parse(multipart: &mut Multipart) -> Result<Self, Error> {
		let Some(field) = multipart.next_field().await? else {
			return Err(MultipartParseError::MissingField {
				expected_field: "image or url".to_string(),
			}
			.into());
		};

		Self::from_field(field).await
	}

	pub async fn from_field(field: Field<'_>) -> Result<Self, Error> {
		let Some(name) = field.name() else {
			return Err(MultipartParseError::NamelessField.into());
		};

		let image = match name {
			"image" => {
				let bytes = field.bytes().await?;

				Self::Image(bytes)
			},
			"url" => {
				let text = field.text().await?;

				Self::Url(text)
			},
			n => {
				return Err(MultipartParseError::UnknownField {
					field_name: n.to_string(),
				}
				.into());
			},
		};

		Ok(image)
	}
}

impl From<CreateImageRequest> for ImageVariant {
	fn from(value: CreateImageRequest) -> Self {
		match value {
			CreateImageRequest::Url(f) => Self::Url(f),
			CreateImageRequest::Image(b) => Self::Image(b),
		}
	}
}

#[derive(Clone, Debug)]
pub struct CreateOrderedImageRequest {
	pub image: CreateImageRequest,
	pub index: i32,
}

impl CreateOrderedImageRequest {
	pub async fn parse(multipart: &mut Multipart) -> Result<Self, Error> {
		let image = CreateImageRequest::parse(multipart).await?;

		let Some(field) = multipart.next_field().await? else {
			return Err(MultipartParseError::MissingField {
				expected_field: "index".to_string(),
			}
			.into());
		};

		let Some(name) = field.name() else {
			return Err(MultipartParseError::NamelessField.into());
		};

		if name != "index" {
			return Err(MultipartParseError::UnknownField {
				field_name: name.to_string(),
			}
			.into());
		}

		let name = name.to_string();

		let index: i32 = field.text().await?.parse().map_err(|_| {
			MultipartParseError::WrongFieldType {
				field_name:  name,
				expected_ty: "number".to_string(),
			}
		})?;

		Ok(Self { image, index })
	}
}

impl From<CreateOrderedImageRequest> for OrderedImageVariant {
	fn from(value: CreateOrderedImageRequest) -> Self {
		let image = ImageVariant::from(value.image);

		Self { image, index: value.index }
	}
}
