#[macro_use]
extern crate tracing;

use std::time::Duration;

use axum::Router;
use axum::routing::{delete, get, post};
use deadpool_diesel::postgres::{Object, Pool};

mod config;
pub mod controllers;
pub mod error;
pub mod models;
pub mod schema;

pub type DbPool = Pool;
pub type DbConn = Object;

pub use config::Config;
use controllers::healthcheck;
use controllers::profile::get_all_profiles;
use controllers::translation::{
	create_translation,
	create_translations,
	delete_translation,
	delete_translations,
	get_translation,
	get_translations,
};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

/// Create an axum app
///
/// # Panics
/// Panics if configuration fails
pub fn create_app(config: Config, db_pool: DbPool) -> Router {
	Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest("/profile", Router::new().route("/", get(get_all_profiles)))
		.nest(
			"/translation",
			Router::new()
				.route("/", post(create_translation))
				.route("/bulk", post(create_translations))
				.route("/{key}", get(get_translations))
				.route("/{key}/{language}", get(get_translation))
				.route("/{key}", delete(delete_translations))
				.route("/{key}/{language}", delete(delete_translation)),
		)
		.layer(TraceLayer::new_for_http())
		.layer(TimeoutLayer::new(Duration::from_secs(5)))
		.with_state(db_pool)
		.with_state(config)
}
