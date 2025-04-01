use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use axum_test::TestServer;
use blokmap::mailer::{Mailer, StubMailbox};
use blokmap::{AppState, Config, SeedProfile, Seeder, routes};
use mock_redis::{RedisUrlGuard, RedisUrlProvider};

pub mod wrappers;

mod mock_db;
mod mock_redis;

use mock_db::{DATABASE_PROVIDER, DatabaseGuard};

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

		let test_pool_guard = (*DATABASE_PROVIDER).acquire().await;
		let test_pool = test_pool_guard.create_pool();

		{
			let conn = test_pool.get().await.unwrap();
			let seeder = Seeder::new(&conn);

			seeder
				.populate("seed/profiles.json", async |conn, profiles| {
					for profile in profiles {
						SeedProfile::insert(profile, conn).await?;
					}

					Ok(())
				})
				.await;
		}

		let redis_url_guard = RedisUrlProvider::acquire();
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
}
