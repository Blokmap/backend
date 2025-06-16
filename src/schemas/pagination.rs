use serde::{Deserialize, Deserializer, Serialize};

use crate::schemas::BoundedU32Visitor;

const fn page_default() -> u32 { 1 }

const fn per_page_default() -> u32 { 12 }

/// Pagination request parameters.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
	#[serde(default = "page_default", deserialize_with = "ds_page_bounds")]
	pub page:     u32,
	#[serde(
		default = "per_page_default",
		deserialize_with = "ds_per_page_bounds"
	)]
	pub per_page: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationResponse<T> {
	pub page:     u32,
	pub per_page: u32,
	pub total:    i64,
	pub data:     T,
}

impl Default for PaginationOptions {
	fn default() -> Self { Self { page: 1, per_page: 12 } }
}

impl PaginationOptions {
	/// Create a new [`Paginated`] struct based on the current parameters with
	/// the given data
	pub fn paginate<T>(&self, total: i64, data: T) -> PaginationResponse<T> {
		PaginationResponse {
			page: self.page,
			per_page: self.per_page,
			total,
			data,
		}
	}

	/// Calculate the SQL LIMIT value of these parameters
	#[inline]
	#[must_use]
	pub fn limit(&self) -> i64 { self.per_page.into() }

	/// Calculate the SQL OFFSET value of these parameters
	#[inline]
	#[must_use]
	pub fn offset(&self) -> i64 { ((self.page - 1) * self.per_page).into() }
}

/// Deserialization visitor for `page` bounds.
fn ds_page_bounds<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
	d.deserialize_u32(BoundedU32Visitor { start: 1, end: u32::MAX })
}

/// Deserialization visitor for `per_page` bounds.
fn ds_per_page_bounds<'de, D: Deserializer<'de>>(
	d: D,
) -> Result<u32, D::Error> {
	d.deserialize_u32(BoundedU32Visitor { start: 1, end: 50 })
}
