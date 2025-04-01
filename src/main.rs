// ▼▼ Required for alpine builds to work ▼▼
extern crate openssl;
#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
// ▲▲ Required for alpine builds to work ▲▲

#[macro_use]
extern crate tracing;

use axum_extra::extract::cookie::Key;
use blokmap::mailer::Mailer;
use blokmap::{AppState, Config, routes};
#[cfg(feature = "seeder")]
use blokmap::{SeedProfile, Seeder};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::signal::unix::SignalKind;
use tracing::Level;

#[tokio::main]
async fn main() {
	// Set up the tracing subscriber.
	// This will print out all logs to the console.
	tracing_subscriber::fmt()
		.pretty()
		.with_thread_names(true)
		.with_max_level(Level::DEBUG)
		.init();

	// Load the configuration from the environment,
	// and create a database pool.
	let config = Config::from_env();
	let database_pool = config.create_database_pool();
	let redis_connection = config.create_redis_connection().await;

	#[cfg(feature = "seeder")]
	{
		let conn = database_pool.get().await.unwrap();
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

	let cookie_jar_key = Key::from(
		&std::fs::read("/run/secrets/cookie-jar-key")
			.expect("COULD NOT READ COOKIE JAR KEY"),
	);

	let stub_mailbox = config.create_stub_mailbox();

	let mailer = Mailer::new(&config, stub_mailbox);

	// Crate the app router and listener.
	let router = routes::get_app_router(AppState {
		config,
		database_pool,
		redis_connection,
		cookie_jar_key,
		mailer,
	});
	let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();

	// Start the server.
	debug!("listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, router)
		.with_graceful_shutdown(shutdown_handler())
		.await
		.unwrap();
}

/// Gracefully shutdown the server on SIGINT or SIGTERM.
async fn shutdown_handler() {
	let ctrl_c = async {
		signal::ctrl_c().await.expect("COULD NOT INSTALL CTRL+C HANDLER");
	};

	let terminate = async {
		signal::unix::signal(SignalKind::terminate())
			.expect("COULD NOT INSTALL TERMINATE SIGNAL HANDLER")
			.recv()
			.await;
	};

	tokio::select! {
		() = ctrl_c => {},
		() = terminate => {},
	}
}
