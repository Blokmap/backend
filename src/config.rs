use std::sync::Arc;

use chrono::Duration;
use deadpool_diesel::postgres::{Manager, Pool};
use lettre::Address;
use openidconnect::{ClientId, ClientSecret};

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

	pub production: bool,

	pub frontend_url: String,

	pub email_confirmation_token_lifetime: Duration,
	pub password_reset_token_lifetime:     Duration,

	pub access_token_name:     String,
	pub access_token_lifetime: time::Duration,

	pub refresh_token_name:     String,
	pub refresh_token_lifetime: time::Duration,

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

		let frontend_url = get_env("FRONTEND_URL");

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

		let access_token_name =
			get_env_default("ACCESS_TOKEN_NAME", "blokmap_access_token");

		let access_token_lifetime = time::Duration::minutes(
			get_env_default("ACCESS_TOKEN_LIFETIME_MINUTES", "10")
				.parse::<i64>()
				.unwrap(),
		);

		let refresh_token_name =
			get_env_default("REFRESH_TOKEN_NAME", "blokmap_refresh_token");

		let refresh_token_lifetime = time::Duration::minutes(
			get_env_default("REFRESH_TOKEN_LIFETIME_MINUTES", "10080") // 1 week
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
			frontend_url,
			email_confirmation_token_lifetime,
			password_reset_token_lifetime,
			access_token_name,
			access_token_lifetime,
			refresh_token_name,
			refresh_token_lifetime,
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
			self.database_url.to_string(),
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

#[derive(Debug, Clone)]
pub struct SsoConfig {
	pub google_client_id:     ClientId,
	pub google_client_secret: ClientSecret,
}

impl SsoConfig {
	/// Create a new [`SsoConfig`] from environment variables
	///
	/// # Panics
	/// Panics if a required environment variable is missing
	#[must_use]
	pub fn from_env() -> Self {
		let google_oidc_credentials =
			std::fs::read_to_string("/run/secrets/google-oidc-credentials")
				.expect("GOOGLE OIDC CREDENTIALS MISSING");

		let google_oidc_credentials =
			google_oidc_credentials.lines().collect::<Vec<_>>();

		assert_eq!(google_oidc_credentials.len(), 2);

		let google_client_id =
			ClientId::new(google_oidc_credentials[0].to_owned());
		let google_client_secret =
			ClientSecret::new(google_oidc_credentials[1].to_owned());

		Self { google_client_id, google_client_secret }
	}

	/// Create a new [`SsoConfig`] with all fields empty to be used in tests
	#[must_use]
	pub fn stub() -> Self {
		Self {
			google_client_id:     ClientId::new(String::new()),
			google_client_secret: ClientSecret::new(String::new()),
		}
	}
}
