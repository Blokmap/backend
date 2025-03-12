use std::sync::Arc;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use axum_extra::extract::cookie::Key;
use axum_test::TestServer;
use blokmap::mailer::{Mailer, StubMailbox};
use blokmap::{AppState, Config, routes};
use mock_redis::{RedisUrlGuard, RedisUrlLock};

pub mod wrappers;

mod mock_db;
mod mock_redis;

use mock_db::{DatabaseGuard, TEST_DATABASE_FIXTURE};

#[allow(dead_code)]
pub struct TestEnv {
	pub app:          TestServer,
	pub db_guard:     DatabaseGuard,
	pub redis_guard:  RedisUrlGuard,
	pub stub_mailbox: Arc<StubMailbox>,
}

impl TestEnv {
	/// Get a test environment with mocked resources for running tests
	///
	/// # Panics
	/// Panics if building a test server or mailbox fails
	pub async fn new() -> Self {
		let config = Config::from_env();

		let test_pool_guard = (*TEST_DATABASE_FIXTURE).acquire().await;
		let test_pool = test_pool_guard.create_pool();

		let redis_url_guard = RedisUrlLock::get();
		let redis_connection = redis_url_guard.connect().await;

		let cookie_jar_key = Key::from(&[0u8; 64]);

		let stub_mailbox = config.create_stub_mailbox();

		let mailer = Mailer::new(&config, stub_mailbox.clone());

		let state = AppState {
			config,
			database_pool: test_pool.clone(),
			redis_connection,
			cookie_jar_key,
			mailer,
		};
		let app = routes::get_app_router(state);

		let test_server =
			TestServer::builder().save_cookies().build(app).unwrap();

		TestEnv {
			app:          test_server,
			db_guard:     test_pool_guard,
			redis_guard:  redis_url_guard,
			stub_mailbox: stub_mailbox.unwrap(),
		}
	}

	/// Create a test user in the test environment
	///
	/// # Panics
	/// Panics if creating the user fails for any reason
	pub async fn create_test_user(self) -> Self {
		let salt = SaltString::generate(&mut OsRng);
		let password_hash = Argon2::default()
			.hash_password("bobdebouwer1234!".as_bytes(), &salt)
			.unwrap()
			.to_string();

		let pool = self.db_guard.create_pool();
		let conn = pool.get().await.unwrap();

		conn.interact(|conn| {
			use diesel::prelude::*;
			use diesel::sql_types::Text;

			diesel::sql_query(
				"INSERT INTO profile (username, password_hash, email, state) \
				 VALUES ('bob', $1, 'bob@example.com', 'active');",
			)
			.bind::<Text, _>(password_hash)
			.execute(conn)
		})
		.await
		.unwrap()
		.unwrap();

		self
	}
}
