use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::controllers::location::{create_location, delete_location, get_location, get_location_positions, get_locations, update_location};
use crate::AppState;
use crate::controllers::auth::{
	confirm_email,
	login_profile_with_email,
	login_profile_with_username,
	logout_profile,
	register_profile,
	request_password_reset,
	resend_confirmation_email,
	reset_password,
};
use crate::controllers::healthcheck;
use crate::controllers::profile::{
	activate_profile,
	disable_profile,
	get_all_profiles,
	get_current_profile,
	update_current_profile,
};

use crate::controllers::translation::{
	create_translation,
	delete_translation,
	get_translation,
	update_translation,
};
use crate::middleware::{AdminLayer, AuthLayer};

/// Get the app router.
pub fn get_app_router(state: AppState) -> Router {
	let api_routes = Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest(
			"/translations",
			get_translation_routes().route_layer(AuthLayer::new(state.clone())),
		)
		.nest("/locations", get_location_routes())
		.nest("/auth", get_auth_routes(&state))
		.nest(
			"/profile",
			get_profile_routes(&state)
				.route_layer(AuthLayer::new(state.clone())),
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

/// Get the auth routes.
fn get_auth_routes(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/register", post(register_profile))
		.route("/confirm_email/{token}", post(confirm_email))
		.route(
			"/resend_confirmation_email/{token}",
			post(resend_confirmation_email),
		)
		.route("/request_password_reset", post(request_password_reset))
		.route("/reset_password", post(reset_password))
		.route("/login/username", post(login_profile_with_username))
		.route("/login/email", post(login_profile_with_email))
		.route(
			"/logout",
			post(logout_profile).route_layer(AuthLayer::new(state.clone())),
		)
}

fn get_profile_routes(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/", get(get_all_profiles))
		.route("/me", get(get_current_profile).patch(update_current_profile))
		.merge(
			Router::new()
				.route("/disable/{profile_id}", post(disable_profile))
				.route("/activate/{profile_id}", post(activate_profile))
				.route_layer(AdminLayer::new(state.clone())),
		)
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
	Router::new()
		.route("/", post(create_location).get(get_locations))
		.route("/positions", get(get_location_positions))
		.route(
			"/{id}",
			get(get_location).post(update_location).delete(delete_location),
		)
}
