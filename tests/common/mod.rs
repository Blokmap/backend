use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use axum_test::TestServer;
use blokmap::controllers::auth::LoginUsernameRequest;
use blokmap::mailer::{Mailer, StubMailbox};
use blokmap::models::{
	Location,
	NewLocation,
	NewTranslation,
	Profile,
	Translation,
};
use blokmap::{AppState, Config, Error, SeedProfile, Seeder, routes};
use mock_redis::{RedisUrlGuard, RedisUrlProvider};

mod wrap_mail;

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
		// Load the configuration from the environment
		let config = Config::from_env();

		// Create a test database pool
		let test_pool_guard = (*DATABASE_PROVIDER).acquire().await;
		let test_pool = test_pool_guard.create_pool();

		// Run the seeders to populate the test database
		{
			let conn = test_pool.get().await.unwrap();
			let seeder = Seeder::new(&conn);

			// Seed profiles
			seeder
				.populate("tests/seed/profiles.json", async |conn, profiles| {
					for profile in profiles {
						SeedProfile::insert(profile, conn).await?;
					}

					Ok(())
				})
				.await;

			// Seed translations
			seeder
				.populate(
					"tests/seed/translations.json",
					async |conn, translations: Vec<NewTranslation>| {
						for translation in translations {
							translation.insert(conn).await?;
						}

						Ok(())
					},
				)
				.await;

			// Seed locations
			seeder
				.populate(
					"tests/seed/locations.json",
					async |conn, locations: Vec<NewLocation>| {
						for location in locations {
							location.insert(conn).await?;
						}

						Ok(())
					},
				)
				.await;
		}

		// Create a test Redis connection
		let redis_url_guard = RedisUrlProvider::acquire();
		let redis_connection = redis_url_guard.connect().await;

		// Create a cookie jar key
		let cookie_jar_key = Key::from(&[0u8; 64]);

		// Create a stub mailbox
		let stub_mailbox = config.create_stub_mailbox();

		// Create a test Mailer
		let mailer = Mailer::new(&config, stub_mailbox.clone());

		// Create the test app.
		let app = routes::get_app_router(AppState {
			config,
			database_pool: test_pool.clone(),
			redis_connection,
			cookie_jar_key,
			mailer,
		});

		let test_server =
			TestServer::builder().save_cookies().build(app).unwrap();

		TestEnv {
			app:          test_server,
			db_guard:     test_pool_guard,
			redis_guard:  redis_url_guard,
			stub_mailbox: stub_mailbox.unwrap(),
		}
	}

	/// Login as a test user
	/// These assume the seeders have been run and the test user exists
	#[allow(dead_code)]
	pub async fn login(self, username: &str) -> Self {
		self.app
			.post("/auth/login/username")
			.json(&LoginUsernameRequest {
				username: username.to_string(),
				password: "foo".to_string(),
			})
			.await;

		self
	}

	/// Login as a test admin
	/// These assume the seeders have been run and the test user exists
	#[allow(dead_code)]
	pub async fn login_admin(self) -> Self { self.login("test-admin").await }
}

impl TestEnv {
	/// Get a test user profile from the test database
	#[allow(dead_code)]
	pub async fn get_profile(&self, username: &str) -> Result<Profile, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		let profile =
			Profile::get_by_username(username.to_string(), &conn).await?;
		Ok(profile)
	}

	/// Get a test admin profile from the test database
	#[allow(dead_code)]
	pub async fn get_admin_profile(&self) -> Result<Profile, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		let profile =
			Profile::get_by_username("test-admin".to_string(), &conn).await?;
		Ok(profile)
	}

	/// Get a test translation in the test database
	#[allow(dead_code)]
	pub async fn get_translation(&self) -> Result<Translation, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		Translation::get_by_id(1, &conn).await
	}

	/// Get a location from the test database
	#[allow(dead_code)]
	pub async fn get_location(&self) -> Result<Location, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		let (location, ..) = Location::get_by_id(1, &conn).await?;
		Ok(location)
	}
}
