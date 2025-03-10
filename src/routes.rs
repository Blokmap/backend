use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::controllers::healthcheck;
use crate::controllers::location::{
	create_location,
	delete_location,
	get_location,
	get_locations,
	update_location,
};
use crate::controllers::profile::get_all_profiles;
use crate::controllers::translation::{
	create_translation,
	delete_translation,
	get_translation,
	update_translation,
};

/// Get the app router.
pub fn get_app_router(state: AppState) -> Router {
	let api_routes = Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest("/profile", get_profile_routes())
		.nest("/translation", get_translation_routes())
		.nest("/location", get_location_routes());

	Router::new()
		.merge(api_routes)
		.layer(TraceLayer::new_for_http())
		.layer(TimeoutLayer::new(Duration::from_secs(5)))
		.with_state(state)
}

/// Get the profile routes.
fn get_profile_routes() -> Router<AppState> {
	Router::new().route("/", get(get_all_profiles))
}

/// Get the translation routes.
fn get_translation_routes() -> Router<AppState> {
	Router::new().route("/", post(create_translation)).route(
		"/{id}",
		get(get_translation)
			.delete(delete_translation)
			.post(update_translation),
	)
}

/// Get the location routes.
fn get_location_routes() -> Router<AppState> {
	Router::new().route("/", post(create_location).get(get_locations)).route(
		"/{id}",
		get(get_location).post(update_location).delete(delete_location),
	)
}
