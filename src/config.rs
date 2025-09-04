use std::sync::Arc;

use chrono::Duration;
use deadpool_diesel::postgres::{Manager, Pool};
use lettre::Address;
use url::Url;

use crate::RedisConn;
use crate::mailer::StubMailbox;

/// Get an environment variable or panic if it is not set.
fn get_env(var: &str) -> String {
	std::env::var(var).unwrap_or_else(|_| panic!("{var} must be set"))
}

/// Get an environment variable or use a default value.
fn get_env_default(var: &str, default: impl Into<String>) -> String {
	std::env::var(var).unwrap_or_else(|_| {
		warn!("{var} not set, using default");

		default.into()
	})
}

/// Configuration settings for the application
#[derive(Clone, Debug)]
pub struct Config {
	pub database_url: String,
	pub redis_url:    String,

	pub production:  bool,
	pub skip_verify: bool,

	pub backend_url:  Url,
	pub frontend_url: Url,
	pub static_url:   Url,

	pub email_confirmation_token_lifetime: Duration,
	pub password_reset_token_lifetime:     Duration,

	pub claims_cookie_name:     String,
	pub access_cookie_name:     String,
	pub access_cookie_lifetime: time::Duration,

	pub email_address:       Address,
	pub email_queue_size:    usize,
	pub email_smtp_server:   String,
	pub email_smtp_password: String,
}

impl Config {
	/// Create a new [`Config`] from environment variables
	///
	/// # Panics
	/// Panics if a required environment variable is missing
	#[must_use]
	pub fn from_env() -> Self {
		let database_url = get_env("DATABASE_URL");
		let redis_url = get_env("REDIS_URL");

		let production =
			get_env_default("PRODUCTION", "false").parse::<bool>().unwrap();

		let skip_verify =
			get_env_default("SKIP_VERIFY", "true").parse::<bool>().unwrap();

		let backend_url =
			get_env("BACKEND_URL").parse().expect("INVALID BASE URL");
		let frontend_url =
			get_env("FRONTEND_URL").parse().expect("INVALID FRONTEND URL");
		let static_url =
			get_env("STATIC_URL").parse().expect("INVALID STATIC URL");

		let email_confirmation_token_lifetime = Duration::minutes(
			get_env_default("EMAIL_CONFIRMATION_TOKEN_LIFETIME", "5")
				.parse::<i64>()
				.unwrap(),
		);
		let password_reset_token_lifetime = Duration::minutes(
			get_env_default("PASSWORD_RESET_TOKEN_LIFETIME", "5")
				.parse::<i64>()
				.unwrap(),
		);

		let claims_cookie_name =
			get_env_default("CLAIMS_COOKIE_NAME", "blokmap_login_claims");

		let access_cookie_name =
			get_env_default("ACCESS_COOKIE_NAME", "blokmap_access_token");

		let access_cookie_lifetime = time::Duration::minutes(
			get_env_default("ACCESS_COOKIE_LIFETIME_MINUTES", "120")
				.parse::<i64>()
				.unwrap(),
		);

		let email_address =
			get_env_default("EMAIL_ADDRESS", "blokmap@gmail.com")
				.parse::<Address>()
				.expect("INVALID EMAIL ADDRESS");

		let email_queue_size = get_env_default("EMAIL_QUEUE_SIZE", "32")
			.parse::<usize>()
			.expect("INVALID EMAIL QUEUE SIZE");

		let email_smtp_server = get_env_default("EMAIL_SMTP_SERVER", "stub");

		let email_smtp_password =
			std::fs::read_to_string("/run/secrets/smtp-password")
				.unwrap_or_else(|_| {
					warn!("SMTP PASSWORD not set, using default");

					String::new()
				});

		Self {
			database_url,
			redis_url,
			production,
			skip_verify,
			backend_url,
			frontend_url,
			static_url,
			email_confirmation_token_lifetime,
			password_reset_token_lifetime,
			claims_cookie_name,
			access_cookie_name,
			access_cookie_lifetime,
			email_address,
			email_queue_size,
			email_smtp_server,
			email_smtp_password,
		}
	}

	/// Create a database pool for the given config
	///
	/// # Panics
	/// Panics if creating the pool fails
	#[must_use]
	pub fn create_database_pool(&self) -> Pool {
		let manager = Manager::new(
			self.database_url.clone(),
			deadpool_diesel::Runtime::Tokio1,
		);

		Pool::builder(manager).build().unwrap()
	}

	/// Create a stub mailbox based on the current config
	#[must_use]
	pub fn create_stub_mailbox(&self) -> Option<Arc<StubMailbox>> {
		if self.email_smtp_server != "stub" {
			return None;
		}

		Some(Arc::new(StubMailbox::default()))
	}

	/// Create a connection to the cache
	///
	/// # Panics
	/// Panics if connecting fails
	pub async fn create_redis_connection(&self) -> RedisConn {
		let client = redis::Client::open(self.redis_url.as_str())
			.expect("COULD NOT CREATE REDIS CLIENT");

		client
			.get_multiplexed_async_connection()
			.await
			.expect("COULD NOT CONNECT TO REDIS")
	}
}
