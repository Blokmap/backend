// ▼▼ Required for alpine builds to work ▼▼
extern crate openssl;
#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
// ▲▲ Required for alpine builds to work ▲▲

#[macro_use]
extern crate tracing;

use blokmap_backend::{Config, create_app};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::signal::unix::SignalKind;
use tracing::Level;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt()
		.pretty()
		.with_thread_names(true)
		.with_max_level(Level::DEBUG)
		.init();

	// Set up the configuration.
	let config = Config::from_env();

	// Set up the database connection pool.
	let pool = config.create_database_pool();

	let app = create_app(&config, pool);

	let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
	debug!("listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_handler())
		.await
		.unwrap();
}

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
