use chrono::TimeDelta;
use deadpool_diesel::postgres::{Manager, Pool};

/// Configuration settings for the application
#[derive(Clone, Debug)]
pub struct Config {
	pub database_url: String,

	pub access_token_name:     String,
	pub access_token_lifetime: TimeDelta,
}

impl Config {
	fn get_env(var: &str) -> String {
		std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"))
	}

	fn get_env_default(var: &str, default: impl Into<String>) -> String {
		std::env::var(var).unwrap_or_else(|_| {
			warn!("{var} not set, using default");

			default.into()
		})
	}

	/// Create a new [`Config`] from environment variables
	///
	/// # Panics
	/// Panics if an environment variable is missing
	#[must_use]
	pub fn from_env() -> Self {
		let database_url = Self::get_env("DATABASE_URL");

		let access_token_name =
			Self::get_env_default("ACCESS_TOKEN_NAME", "access_token");
		let access_token_lifetime = TimeDelta::minutes(
			Self::get_env_default("ACCESS_TOKEN_LIFETIME_MINUTES", "10")
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
