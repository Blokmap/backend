use std::path::PathBuf;

use diesel::prelude::*;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::models::{Profile, ProfileState};
use crate::{DbConn, Error};

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

		serde_json::from_str(&s)
			.unwrap_or_else(|_| panic!("COULD NOT MAP SEED FILE {filename}"))
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
		T: DeserializeOwned + std::fmt::Debug,
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
	///
	/// # Errors
	/// Errors if the password is invalid or if interacting with the database
	/// fails
	pub async fn insert(self, conn: &DbConn) -> Result<(), Error> {
		let hash = Profile::hash_password(&self.password)?;
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
				.values(insertable)
				.on_conflict_do_nothing()
				.execute(conn)
		})
		.await??;

		Ok(())
	}
}
