use std::collections::HashSet;

use clap::Error;
use clap::error::ErrorKind;
use common::DbConn;
use diesel::PgConnection;
use rand::distr::Alphanumeric;
use rand::{Rng, rng};

/// Generate a unique set of items using a closure
pub fn generate_unique_set<F>(count: usize, mut f: F) -> Vec<String>
where
	F: FnMut() -> String,
{
	let mut set = HashSet::with_capacity(count);
	let mut rng = rng();

	while set.len() < count {
		let mut value = f();
		value.push('_');
		value.extend((0..6).map(|_| rng.sample(Alphanumeric) as char));
		set.insert(value);
	}

	set.into_iter().collect()
}

/// Generic batch insertion function using closure-based approach
pub async fn batch_insert<T, F>(
	conn: &DbConn,
	mut items: Vec<T>,
	chunk_size: usize,
	inserter: F,
) -> Result<usize, Error>
where
	T: Send + 'static,
	F: Fn(&mut PgConnection, &[T]) -> Result<usize, diesel::result::Error>
		+ Send
		+ Copy
		+ 'static,
{
	let size = items.len();
	let mut total = 0;

	while !items.is_empty() {
		let chunk =
			items.drain(..chunk_size.min(items.len())).collect::<Vec<_>>();
		let chunk_len = chunk.len();

		let insert_len = conn
			.interact(move |c| inserter(c, &chunk))
			.await
			.map_err(|e| Error::raw(ErrorKind::Io, e))?
			.map_err(|e| Error::raw(ErrorKind::Io, e))?;

		total += insert_len;

		println!("Inserted {total}/{size} items");

		if insert_len != chunk_len {
			return Err(Error::raw(
				ErrorKind::Io,
				format!("Inserted {insert_len} items but expected {chunk_len}"),
			));
		}
	}

	Ok(total)
}
