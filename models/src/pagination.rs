use std::fmt;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
	pub page:     u32,
	pub per_page: u32,
	pub total:    i64,

	pub data: T,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
	#[serde(default = "page_default", deserialize_with = "page_bounds")]
	pub page:     u32,
	#[serde(
		default = "per_page_default",
		deserialize_with = "per_page_bounds"
	)]
	pub per_page: u32,
}

const fn page_default() -> u32 { 1 }

const fn per_page_default() -> u32 { 12 }

struct BoundedU32Visitor {
	start: u32,
	end:   u32,
}

impl Visitor<'_> for BoundedU32Visitor {
	type Value = u32;

	fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "a number between {} and {}", self.start, self.end)
	}

	fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		if (self.start..=self.end).contains(&v) {
			Ok(v)
		} else {
			Err(E::custom(format_args!(
				"invalid value: {v}, expected a number between {} and {}",
				self.start, self.end,
			)))
		}
	}
}

fn page_bounds<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
	d.deserialize_u32(BoundedU32Visitor { start: 1, end: u32::MAX })
}

fn per_page_bounds<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
	d.deserialize_u32(BoundedU32Visitor { start: 1, end: 75 })
}

impl Default for PaginationOptions {
	fn default() -> Self { Self { page: 1, per_page: 12 } }
}

impl PaginationOptions {
	/// Create a new [`Paginated`] struct based on the current parameters with
	/// the given data
	pub fn paginate<T>(&self, total: i64, data: T) -> Paginated<T> {
		Paginated { page: self.page, per_page: self.per_page, total, data }
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
