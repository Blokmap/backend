use chrono::NaiveDateTime;
use common::Error;
use models::{NewReview, Review, ReviewUpdate, SimpleProfile};
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator_derive::Validate;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewResponse {
	pub id:         i32,
	pub created_by: SimpleProfile,
	pub rating:     i32,
	pub body:       Option<String>,
	pub created_at: NaiveDateTime,
	pub updated_at: NaiveDateTime,
}

impl From<Review> for ReviewResponse {
	fn from(value: Review) -> Self {
		Self {
			id:         value.review.id,
			created_by: value.created_by,
			rating:     value.review.rating,
			body:       value.review.body,
			created_at: value.review.created_at,
			updated_at: value.review.updated_at,
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateReviewRequest {
	#[validate(range(min = 0, max = 5))]
	pub rating: i32,
	pub body:   Option<String>,
}

impl CreateReviewRequest {
	pub fn to_insertable(
		self,
		profile_id: i32,
		location_id: i32,
	) -> Result<NewReview, Error> {
		self.validate()?;

		Ok(NewReview {
			profile_id,
			location_id,
			rating: self.rating,
			body: self.body,
		})
	}
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReviewRequest {
	#[validate(range(min = 0, max = 5))]
	pub rating: Option<i32>,
	pub body:   Option<String>,
}

impl UpdateReviewRequest {
	pub fn to_insertable(self) -> Result<ReviewUpdate, Error> {
		self.validate()?;

		Ok(ReviewUpdate { rating: self.rating, body: self.body })
	}
}
