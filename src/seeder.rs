use std::path::PathBuf;

use common::{DbConn, Error};
use diesel::prelude::*;
use models::ProfileState;
use serde::Deserialize;
use serde::de::DeserializeOwned;

pub struct Seeder<'c> {
	connection: &'c DbConn,
}

impl<'c> Seeder<'c> {
	#[must_use]
	pub fn new(connection: &'c DbConn) -> Self { Self { connection } }

	/// Read a file into a series of deserializable items
	///
	/// # Panics
	/// Panics if reading or deserializing the file fails
	fn read_file_records<T, I>(filename: &str) -> I
	where
		T: DeserializeOwned,
		I: IntoIterator<Item = T> + DeserializeOwned,
	{
		let path = std::env::var("CARGO_MANIFEST_DIR")
			.map(PathBuf::from)
			.unwrap_or_default()
			.join(filename);

		let s = std::fs::read_to_string(path)
			.unwrap_or_else(|_| panic!("COULD NOT READ SEED FILE {filename}"));

		serde_json::from_str(&s).unwrap_or_else(|e| {
			panic!("COULD NOT MAP SEED FILE {filename}\n{e:?}")
		})
	}

	/// Load a file and populate the database with it
	///
	/// # Panics
	/// Panics if reading the file or interacting with the database fails
	pub async fn populate<'s, T, F>(
		&'s self,
		filename: &str,
		loader: F,
	) -> &'s Self
	where
		T: DeserializeOwned,
		F: AsyncFnOnce(&DbConn, Vec<T>) -> Result<(), Error>,
	{
		let records = Self::read_file_records(filename);

		loader(self.connection, records).await.unwrap_or_else(|e| {
			panic!("COULD NOT LOAD RECORDS FOR {filename}\n{e:?}")
		});

		info!("seeded database from {filename}");

		self
	}
}

#[derive(Clone, Debug, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = models::schema::profile)]
pub struct SeedProfile {
	username:      String,
	password_hash: String,
	email:         String,
	#[serde(default)]
	is_admin:      bool,
	#[serde(default)]
	state:         ProfileState,
}

impl SeedProfile {
	/// Insert this [`SeedProfile`]
	///
	/// # Errors
	/// Errors if the password is invalid or if interacting with the database
	/// fails
	pub async fn insert(self, conn: &DbConn) -> Result<(), Error> {
		conn.interact(|conn| {
			use models::schema::profile::dsl::*;

			diesel::insert_into(profile)
				.values(self)
				.on_conflict_do_nothing()
				.execute(conn)
		})
		.await??;

		Ok(())
	}
}
