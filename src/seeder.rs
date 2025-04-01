use std::path::PathBuf;

use diesel::prelude::*;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::models::{Profile, ProfileState};
use crate::{DbConn, Error};

pub struct Seeder<'c> {
	connection: &'c mut DbConn,
}

impl<'c> Seeder<'c> {
	pub fn new(connection: &'c mut DbConn) -> Self { Self { connection } }

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
			.expect(&format!("COULD NOT READ SEED FILE {filename}"));

		serde_json::from_str(&s)
			.expect(&format!("COULD NOT MAP SEED FILE {filename}"))
	}

	/// Load a file and populate the database with it
	///
	/// # Panics
	/// Panics if reading the file or interacting with the database fails
	pub async fn populate<'s, T, F>(
		&'s mut self,
		filename: &str,
		loader: F,
	) -> &'s mut Self
	where
		T: DeserializeOwned + std::fmt::Debug,
		F: AsyncFnOnce(&DbConn, Vec<T>) -> Result<(), Error>,
	{
		let records = Self::read_file_records(filename);

		loader(self.connection, records)
			.await
			.expect(&format!("COULD NOT LOAD RECORDS FOR {filename}"));

		info!("seeded database from {filename}");

		self
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct SeedProfile {
	pub username: String,
	pub password: String,
	pub email:    String,
	#[serde(default)]
	pub admin:    bool,
	#[serde(default)]
	pub state:    ProfileState,
}

#[derive(Clone, Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::profile)]
struct InsertableSeedProfile {
	username:      String,
	password_hash: String,
	email:         String,
	admin:         bool,
	state:         ProfileState,
}

impl SeedProfile {
	/// Insert this [`SeedProfile`]
	pub async fn insert(self, conn: &DbConn) -> Result<(), Error> {
		let hash = Profile::hash_password(&self.password).unwrap();
		let insertable = InsertableSeedProfile {
			username:      self.username,
			password_hash: hash,
			email:         self.email,
			admin:         self.admin,
			state:         self.state,
		};

		conn.interact(|conn| {
			use crate::schema::profile::dsl::*;

			diesel::insert_into(profile)
				.values(insertable.clone())
				.on_conflict(username)
				.do_update()
				.set(insertable)
				.execute(conn)
		})
		.await??;

		Ok(())
	}
}
