use std::sync::Arc;

use axum_extra::extract::cookie::Key;
use axum_test::TestServer;
use blokmap::mailer::{Mailer, StubMailbox};
use blokmap::schemas::auth::LoginRequest;
use blokmap::{AppState, Config, SeedProfile, Seeder, SsoConfig, routes};
use common::Error;
use location::{Location, LocationIncludes, NewLocation};
use mock_redis::{RedisUrlGuard, RedisUrlProvider};
use opening_time::{NewOpeningTime, OpeningTime, OpeningTimeIncludes};
use primitives::PrimitiveProfile;
use profile::Profile;
use reservation::NewReservation;
use tag::{NewTag, TagIncludes};
use translation::{NewTranslation, Translation, TranslationIncludes};

mod mock_db;
mod mock_redis;
mod wrap_mail;

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
	#[allow(clippy::too_many_lines)]
	pub async fn new() -> Self {
		// Load the configuration from the environment
		let mut config = Config::from_env();
		let sso_config = SsoConfig::stub();

		config.production = true;
		config.skip_verify = false;

		// Create a test database pool
		tracing::info!("acquiring db guard");
		let test_pool_guard = (*DATABASE_PROVIDER).acquire().await;
		tracing::info!("db guard acquired");
		let test_pool = test_pool_guard.create_pool();

		// Run the seeders to populate the test database
		{
			use diesel::prelude::*;

			let conn = test_pool.get().await.unwrap();
			let seeder = Seeder::new(&conn);

			tracing::info!("seeding database...");

			// Seed profiles
			seeder
				.populate(
					"tests/seed/profiles.json",
					async |conn, records: Vec<SeedProfile>| {
						conn.interact(move |conn| {
							use db::profile::dsl::*;

							diesel::insert_into(profile)
								.values(records)
								.execute(conn)
						})
						.await
						.unwrap()
						.unwrap();

						Ok(())
					},
				)
				.await;

			// Seed translations
			seeder
				.populate(
					"tests/seed/translations.json",
					async |conn, records: Vec<NewTranslation>| {
						conn.interact(move |conn| {
							use db::translation::dsl::*;

							diesel::insert_into(translation)
								.values(records)
								.execute(conn)
						})
						.await
						.unwrap()
						.unwrap();

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
							location
								.insert(LocationIncludes::default(), conn)
								.await?;
						}

						Ok(())
					},
				)
				.await;

			// Seed opening times
			seeder
				.populate(
					"tests/seed/opening-times.json",
					async |conn, records: Vec<NewOpeningTime>| {
						conn.interact(move |conn| {
							use db::opening_time::dsl::*;

							diesel::insert_into(opening_time)
								.values(records)
								.execute(conn)
						})
						.await
						.unwrap()
						.unwrap();

						Ok(())
					},
				)
				.await;

			// Seed tags
			seeder
				.populate(
					"tests/seed/tags.json",
					async |conn, tags: Vec<NewTag>| {
						for tag in tags {
							tag.insert(TagIncludes::default(), conn).await?;
						}

						Ok(())
					},
				)
				.await;

			// Seed reservations
			seeder
				.populate(
					"tests/seed/reservations.json",
					async |conn, records: Vec<NewReservation>| {
						conn.interact(move |conn| {
							use db::reservation::dsl::*;

							diesel::insert_into(reservation)
								.values(records)
								.execute(conn)
						})
						.await
						.unwrap()
						.unwrap();

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
			sso_config,
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
			.post("/auth/login")
			.json(&LoginRequest {
				username: username.to_string(),
				password: "foo".to_string(),
				remember: false,
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
	pub async fn get_profile(
		&self,
		username: &str,
	) -> Result<PrimitiveProfile, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		let profile =
			Profile::get_by_username(username.to_string(), &conn).await?;
		Ok(profile.profile)
	}

	/// Get a test admin profile from the test database
	#[allow(dead_code)]
	pub async fn get_admin_profile(&self) -> Result<PrimitiveProfile, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		let profile =
			Profile::get_by_username("test-admin".to_string(), &conn).await?;
		Ok(profile.profile)
	}

	/// Get a test translation in the test database
	#[allow(dead_code)]
	pub async fn get_translation(&self) -> Result<Translation, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		Translation::get_by_id(1, TranslationIncludes::default(), &conn).await
	}

	/// Get a location from the test database
	#[allow(dead_code)]
	pub async fn get_location(&self) -> Result<Location, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		let (location, ..) =
			Location::get_by_id(1, LocationIncludes::default(), &conn).await?;
		Ok(location)
	}

	/// Get an opening time from the test database
	#[allow(dead_code)]
	pub async fn get_opening_time(&self) -> Result<OpeningTime, Error> {
		let conn = self.db_guard.create_pool().get().await.unwrap();
		OpeningTime::get_by_id(1, OpeningTimeIncludes::default(), &conn).await
	}
}
