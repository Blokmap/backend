use std::collections::HashSet;

use clap::Error;
use clap::error::ErrorKind;
use common::DbConn;
use diesel::PgConnection;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};

/// Generate a unique set of items using a closure - Optimized version
pub fn generate_unique_set<F>(count: usize, mut f: F) -> Vec<String>
where
	F: FnMut() -> String,
{
	let mut set = HashSet::with_capacity(count * 2); // Larger capacity to reduce collisions
	let mut rng = rng();
	let mut results = Vec::with_capacity(count);

	// Pre-generate suffix characters for better performance
	let suffixes: Vec<String> = (0..count * 2)
		.map(|_| (0..8).map(|_| rng.sample(Alphanumeric) as char).collect())
		.collect();

	let mut suffix_idx = 0;

	while results.len() < count && suffix_idx < suffixes.len() {
		let mut value = f();
		value.push('_');
		value.push_str(&suffixes[suffix_idx]);

		if set.insert(value.clone()) {
			results.push(value);
		}
		suffix_idx += 1;
	}

	// Fallback to the original method if we still need more unique values
	while results.len() < count {
		let mut value = f();
		value.push('_');
		value.extend((0..12).map(|_| rng.sample(Alphanumeric) as char)); // Longer suffix

		if set.insert(value.clone()) {
			results.push(value);
		}
	}

	results
}

/// Calculate optimal batch size based on parameter count per item
/// PostgreSQL has a limit of 65,535 parameters per query
pub fn calculate_optimal_batch_size(params_per_item: usize) -> usize {
	const MAX_PARAMS: usize = 65_000; // Leave some buffer
	let optimal_size = MAX_PARAMS / params_per_item;
	// Ensure minimum batch size of 100 for efficiency
	optimal_size.max(100)
}

/// High-performance batch insertion with automatic parameter optimization
pub async fn batch_insert_optimized<T, F>(
	conn: &DbConn,
	mut items: Vec<T>,
	params_per_item: usize,
	inserter: F,
) -> Result<usize, Error>
where
	T: Send + 'static,
	F: Fn(&mut PgConnection, &[T]) -> Result<usize, diesel::result::Error>
		+ Send
		+ Copy
		+ 'static,
{
	if items.is_empty() {
		return Ok(0);
	}

	let optimal_chunk_size = calculate_optimal_batch_size(params_per_item);
	let total_items = items.len();

	let mut total_inserted = 0;
	let mut batch_count = 0;

	while !items.is_empty() {
		let chunk_size = optimal_chunk_size.min(items.len());
		let chunk: Vec<T> = items.drain(..chunk_size).collect();

		let inserted = conn
			.interact(move |c| inserter(c, &chunk))
			.await
			.map_err(|e| Error::raw(ErrorKind::Io, e))?
			.map_err(|e| Error::raw(ErrorKind::Io, e))?;

		total_inserted += inserted;
		batch_count += 1;

		if inserted != chunk_size {
			return Err(Error::raw(
				ErrorKind::Io,
				format!(
					"Batch {}: inserted {} items but expected {}",
					batch_count, inserted, chunk_size
				),
			));
		}
	}

	Ok(total_inserted)
}
