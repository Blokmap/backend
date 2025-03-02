use axum::Router;
use axum::routing::get;
use blokmap_backend::routes::healthcheck;
use blokmap_backend::routes::profile::get_all_profiles;
use deadpool_diesel::postgres::{Manager, Pool};
use tokio::net::TcpListener;
use tracing::Level;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt().pretty().with_thread_names(true).with_max_level(Level::DEBUG).init();

	let db_url = std::env::var("DATABASE_URL").unwrap();

	// set up connection pool
	let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
	let pool = Pool::builder(manager).build().unwrap();

	let app = Router::new()
		.route("/healthcheck", get(healthcheck))
		.route("/profile", get(get_all_profiles))
		.with_state(pool);

	let listener = TcpListener::bind("0.0.0.0:80").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}
