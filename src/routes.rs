use std::time::Duration;

use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use axum::{
	Router,
	routing::{get, post},
};

use crate::AppState;

use crate::controllers::{
	healthcheck,
	profile::get_all_profiles,
	translation::{
		create_bulk_translations, create_translation, delete_bulk_translations,
		delete_translation, get_bulk_translations, get_translation,
	},
};

/// Get the app router.
pub fn get_app_router(state: AppState) -> Router {
	let api_routes = Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest("/profile", get_profile_routes())
		.nest("/translation", get_translation_routes());

	// Return the routes nested with `/api` to make sure
	// that all routes are prefixed with `/api`.
	Router::new()
		.layer(TraceLayer::new_for_http())
		.layer(TimeoutLayer::new(Duration::from_secs(5)))
		.nest("/api/", api_routes)
		.with_state(state)
}

/// Get the profile routes.
fn get_profile_routes() -> Router<AppState> {
	Router::new().route("/", get(get_all_profiles))
}

/// Get the translation routes.
fn get_translation_routes() -> Router<AppState> {
	Router::new()
		.route("/", post(create_translation))
		.route("/bulk", post(create_bulk_translations))
		.route(
			"/{key}",
			get(get_bulk_translations).delete(delete_bulk_translations),
		)
		.route(
			"/{key}/{language}",
			get(get_translation).delete(delete_translation),
		)
}
