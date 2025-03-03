use chrono::TimeDelta;
use deadpool_diesel::postgres::{Manager, Pool};

#[derive(Clone, Debug)]
pub struct Config {
	pub database_url: String,

	pub access_token_name:     String,
	pub access_token_lifetime: TimeDelta,
}

impl Config {
	fn get_env_var(var: &str) -> String {
		std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"))
	}

	/// Create a new [`Config`] from environment variables
	///
	/// # Panics
	/// Panics if an environment variable is missing
	#[must_use]
	pub fn from_env() -> Self {
		let database_url = Self::get_env_var("DATABASE_URL");

		let access_token_name = Self::get_env_var("ACCESS_TOKEN_NAME");
		let access_token_lifetime = TimeDelta::minutes(
			Self::get_env_var("ACCESS_TOKEN_LIFETIME_MINUTES")
				.parse::<i64>()
				.unwrap(),
		);

		Self { database_url, access_token_name, access_token_lifetime }
	}

	/// Create a database pool for the given config
	///
	/// # Panics
	/// Panics if creating the pool fails
	#[must_use]
	pub fn create_database_pool(&self) -> Pool {
		let manager = Manager::new(
			self.database_url.to_string(),
			deadpool_diesel::Runtime::Tokio1,
		);

		Pool::builder(manager).build().unwrap()
	}
}
