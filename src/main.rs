// ▼▼ Required for alpine builds to work ▼▼
extern crate openssl;
#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
// ▲▲ Required for alpine builds to work ▲▲

#[macro_use]
extern crate tracing;

use axum_extra::extract::cookie::Key;
use blokmap::{AppState, Config, routes};
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

	let cookie_jar_key = Key::from(
		&std::fs::read("/run/secrets/cookie-jar-key")
			.expect("COULD NOT READ COOKIE JAR KEY"),
	);

	// Crate the app router and listener.
	let router = routes::get_app_router(AppState {
		config,
		database_pool,
		cookie_jar_key,
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
