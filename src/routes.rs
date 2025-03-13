use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::controllers::auth::{
	confirm_email,
	login_profile_with_email,
	login_profile_with_username,
	logout_profile,
	register_profile,
	resend_confirmation_email,
};
use crate::controllers::healthcheck;
use crate::controllers::profile::{
	get_all_profiles,
	get_current_profile,
	update_current_profile,
};
use crate::controllers::translation::{
	create_bulk_translations,
	create_translation,
	delete_bulk_translations,
	delete_translation,
	get_bulk_translations,
	get_translation,
};
use crate::middleware::AuthLayer;

/// Get the app router.
pub fn get_app_router(state: AppState) -> Router {
	let api_routes = Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest("/auth", get_auth_routes(&state))
		.nest(
			"/profile",
			get_profile_routes().route_layer(AuthLayer::new(state.clone())),
		)
		.nest(
			"/translation",
			get_translation_routes().route_layer(AuthLayer::new(state.clone())),
		);

	Router::new()
		.merge(api_routes)
		.layer(
			ServiceBuilder::new()
				.layer(TraceLayer::new_for_http())
				.layer(TimeoutLayer::new(Duration::from_secs(5)))
				.layer(CompressionLayer::new()),
		)
		.with_state(state)
}

fn get_auth_routes(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/register", post(register_profile))
		.route("/confirm_email/{token}", post(confirm_email))
		.route(
			"/resend_confirmation_email/{token}",
			post(resend_confirmation_email),
		)
		.route("/login/username", post(login_profile_with_username))
		.route("/login/email", post(login_profile_with_email))
		.route(
			"/logout",
			post(logout_profile).route_layer(AuthLayer::new(state.clone())),
		)
}

fn get_profile_routes() -> Router<AppState> {
	Router::new()
		.route("/", get(get_all_profiles))
		.route("/me", get(get_current_profile).patch(update_current_profile))
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
