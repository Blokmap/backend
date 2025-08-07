//! Database model definitions

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate serde_with;
#[macro_use]
extern crate tracing;

use common::{Error, PaginationError};
use diesel::BoxableExpression;
use diesel::pg::Pg;
use diesel::sql_types::{Bool, Nullable};

pub mod db;

mod authority;
mod image;
mod location;
mod opening_time;
mod profile;
mod reservation;
mod review;
mod tag;
mod translation;

pub use authority::*;
pub use image::*;
pub use location::*;
pub use opening_time::*;
pub use profile::*;
pub use reservation::*;
pub use review::*;
pub use tag::*;
pub use translation::*;

const QUERY_HARD_LIMIT: i64 = 1000;

pub type BoxedCondition<S, T = Nullable<Bool>> =
	Box<dyn BoxableExpression<S, Pg, SqlType = T>>;

pub trait ToFilter<S> {
	type SqlType;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType>;
}

#[inline]
fn manual_pagination<T: Clone>(
	items: Vec<T>,
	limit: usize,
	offset: usize,
) -> Result<(usize, bool, Vec<T>), Error> {
	let total = items.len();

	if total == 0 {
		return Ok((total, false, items));
	}

	if offset >= total {
		return Err(PaginationError::OffsetTooLarge.into());
	}

	#[allow(clippy::cast_possible_truncation)]
	let truncated = total == (QUERY_HARD_LIMIT as usize);

	let limit = if limit > items[offset..].len() {
		items[offset..].len()
	} else {
		limit
	};

	let items = items[offset..offset + limit].to_vec();

	Ok((total, truncated, items))
}
