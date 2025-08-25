use common::{Error, PaginationError};
use diesel::BoxableExpression;
use diesel::pg::Pg;
use diesel::sql_types::{Bool, Nullable};

pub const QUERY_HARD_LIMIT: i64 = 100;
pub const RESERVATION_BLOCK_SIZE_MINUTES: i32 = 5;

pub type BoxedCondition<S, T = Nullable<Bool>> =
	Box<dyn BoxableExpression<S, Pg, SqlType = T>>;

pub type PaginatedData<T> = (usize, bool, T);

pub trait ToFilter<S> {
	type SqlType;

	fn to_filter(&self) -> BoxedCondition<S, Self::SqlType>;
}

pub trait JoinParts {
	type Target;
	type Includes;

	fn join(self, includes: Self::Includes) -> Self::Target;
}

#[derive(Clone, Copy, Debug)]
pub struct PaginationConfig {
	pub limit:  usize,
	pub offset: usize,
}

#[inline]
pub fn manual_pagination<T: Clone>(
	items: Vec<T>,
	cfg: PaginationConfig,
) -> Result<PaginatedData<Vec<T>>, Error> {
	let total = items.len();

	if total == 0 {
		let data = (total, false, items);

		return Ok(data);
	}

	if cfg.offset >= total {
		return Err(PaginationError::OffsetTooLarge.into());
	}

	#[allow(clippy::cast_possible_truncation)]
	let truncated = total == (QUERY_HARD_LIMIT as usize);

	let limit = if cfg.limit > items[cfg.offset..].len() {
		items[cfg.offset..].len()
	} else {
		cfg.limit
	};

	let items = items[cfg.offset..cfg.offset + limit].to_vec();

	let data = (total, truncated, items);

	Ok(data)
}
