// ▼▼ Required for alpine builds to work ▼▼
extern crate openssl;
#[allow(unused_imports)]
#[macro_use]
extern crate diesel;
// ▲▲ Required for alpine builds to work ▲▲

#[macro_use]
extern crate tracing;

use std::time::Duration;

use axum::Router;
use axum::routing::get;
use blokmap_backend::config::Config;
use blokmap_backend::controllers::healthcheck;
use blokmap_backend::controllers::profile::get_all_profiles;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::signal::unix::SignalKind;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[tokio::main]
async fn main() {
	// Set up logging.
	tracing_subscriber::fmt().pretty().with_thread_names(true).with_max_level(Level::DEBUG).init();

	// Set up the configuration.
	let config = Config::from_env();

	// Set up the database connection pool.
	let pool = config.setup_database().await;

	let app = Router::new()
		.layer(TraceLayer::new_for_http())
		.layer(TimeoutLayer::new(Duration::from_secs(5)))
		.route("/healthcheck", get(healthcheck))
		.route("/profile", get(get_all_profiles))
		.with_state(pool);

	// Start the server.
	let address = format!("{}:{}", config.server_host, config.server_port);
	let listener = TcpListener::bind(address).await.unwrap();

	debug!("Listening on {}", listener.local_addr().unwrap());
	axum::serve(listener, app).with_graceful_shutdown(shutdown_handler()).await.unwrap();
}

async fn shutdown_handler() {
	let ctrl_c = async {
		signal::ctrl_c().await.expect("COULD NOT INSTALL CTRL+C HANDLER");
	};

	let terminate = async {
		signal::unix::signal(SignalKind::terminate())
			.expect("COULD NOT INSTALL TERMINATE SIGNAL HANDLER")
			.recv()
			.await
	};

	tokio::select! {
		() = ctrl_c => {},
		_ = terminate => {},
	}
}
