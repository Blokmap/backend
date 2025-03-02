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
use blokmap_backend::controllers::healthcheck;
use blokmap_backend::controllers::profile::get_all_profiles;
use deadpool_diesel::postgres::{Manager, Pool};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::signal::unix::SignalKind;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt().pretty().with_thread_names(true).with_max_level(Level::DEBUG).init();

	let db_url = std::env::var("DATABASE_URL").unwrap();

	// set up connection pool
	let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
	let pool = Pool::builder(manager).build().unwrap();

	let app = Router::new()
		.layer(TraceLayer::new_for_http())
		.layer(TimeoutLayer::new(Duration::from_secs(5)))
		.route("/healthcheck", get(healthcheck))
		.route("/profile", get(get_all_profiles))
		.with_state(pool);

	let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
	debug!("listening on {}", listener.local_addr().unwrap());
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
