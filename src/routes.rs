use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::controllers::healthcheck;
use crate::controllers::profile::{
	confirm_email,
	get_all_profiles,
	login_profile_with_email,
	login_profile_with_username,
	logout_profile,
	register_profile,
};
use crate::controllers::translation::{
	create_bulk_translations,
	create_translation,
	delete_bulk_translations,
	delete_translation,
	get_bulk_translations,
	get_translation,
};

/// Get the app router.
pub fn get_app_router(state: AppState) -> Router {
	let api_routes = Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest("/auth", get_auth_routes())
		.nest("/profile", get_profile_routes())
		.nest("/translation", get_translation_routes());

	Router::new()
		.merge(api_routes)
		.layer(TraceLayer::new_for_http())
		.layer(TimeoutLayer::new(Duration::from_secs(5)))
		.with_state(state)
}

fn get_auth_routes() -> Router<AppState> {
	Router::new()
		.route("/register", post(register_profile))
		.route("/confirm_email/{token}", post(confirm_email))
		.route("/login/username", post(login_profile_with_username))
		.route("/login/email", post(login_profile_with_email))
		.route("/logout", post(logout_profile))
}

fn get_profile_routes() -> Router<AppState> {
	Router::new().route("/", get(get_all_profiles))
}

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
