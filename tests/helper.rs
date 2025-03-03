use std::sync::LazyLock;

use axum::Router;
use blokmap_backend::{AppState, Config, DbConn, DbPool, create_app};
use deadpool_diesel::postgres::{Manager, Pool};
use diesel_migrations::{
	EmbeddedMigrations,
	MigrationHarness,
	embed_migrations,
};
use uuid::Uuid;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

/// Global test database provider
pub static TEST_DATABASE_FIXTURE: LazyLock<TestDatabaseFixture> =
	LazyLock::new(|| TestDatabaseFixture::new());

/// A RAII guard provider which generates temporary test databases
pub struct TestDatabaseFixture {
	base_url:  String,
	root_pool: DbPool,
}

/// A test database RAII guard
pub struct DatabaseGuard {
	root_conn:     DbConn,
	database_name: String,
	database_url:  String,
}

/// Get a test axum app with a oneshot database for running tests
pub async fn get_test_app() -> (DatabaseGuard, Router) {
	let config = Config::from_env();

	let test_pool_guard = (*TEST_DATABASE_FIXTURE).acquire().await;
	let test_pool = test_pool_guard.create_pool();

	let state = AppState { config, database_pool: test_pool };

	(test_pool_guard, create_app(state))
}

impl TestDatabaseFixture {
	fn new() -> Self {
		// tracing_subscriber::fmt()
		// 	.pretty()
		// 	.with_thread_names(true)
		// 	.with_max_level(tracing::Level::DEBUG)
		// 	.init();

		let database_url = std::env::var("DATABASE_URL").unwrap();
		let (base_url, _) = database_url.rsplit_once('/').unwrap();
		let base_url = base_url.to_string();

		let manager = Manager::new(
			database_url.to_string(),
			deadpool_diesel::Runtime::Tokio1,
		);

		let root_pool = Pool::builder(manager).build().unwrap();

		Self { base_url, root_pool }
	}

	/// Acquire a new [`DatabaseGuard`] for accessing a temporary test database
	pub async fn acquire(&self) -> DatabaseGuard {
		let uuid = Uuid::new_v4().simple().to_string();
		let database_name = format!("test_{}", uuid);
		let database_url = format!("{}/{}", self.base_url, database_name);

		let root_conn = self
			.root_pool
			.get()
			.await
			.expect("could not get root pool connection");

		let create_db_query = format!("CREATE DATABASE {database_name};");

		root_conn
			.interact(|conn| {
				use diesel::prelude::*;

				diesel::sql_query(create_db_query).execute(conn)
			})
			.await
			.expect("could not interact with root connection")
			.expect("could not create test database");

		DatabaseGuard { root_conn, database_name, database_url }
	}
}

impl DatabaseGuard {
	/// Create a new database pool for this test database guard
	pub fn create_pool(&self) -> DbPool {
		let manager = Manager::new(
			self.database_url.to_string(),
			deadpool_diesel::Runtime::Tokio1,
		);

		let pool = Pool::builder(manager).build().unwrap();

		futures::executor::block_on(async {
			let conn = pool.get().await.unwrap();
			conn.interact(|conn| {
				conn.run_pending_migrations(MIGRATIONS).map(|_| ())
			})
			.await
			.unwrap()
			.unwrap();
		});

		pool
	}
}

impl Drop for DatabaseGuard {
	fn drop(&mut self) {
		let drop_db_query =
			format!("DROP DATABASE {} WITH (FORCE);", self.database_name);

		futures::executor::block_on(async move {
			self.root_conn
				.interact(|conn| {
					use diesel::prelude::*;

					diesel::sql_query(drop_db_query).execute(conn)
				})
				.await
				.expect("could not interact with root connection")
				.expect("could not drop test database");
		});
	}
}
