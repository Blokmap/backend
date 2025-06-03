use std::time::Duration;

use axum::Router;
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::controllers::auth::sso::{sso_callback, sso_login};
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
use crate::controllers::location::{
	approve_location,
	create_location,
	delete_location,
	get_location,
	get_location_positions,
	get_locations,
	search_locations,
	update_location,
	upload_location_image,
};
use crate::controllers::profile::{
	activate_profile,
	disable_profile,
	get_all_profiles,
	get_current_profile,
	get_profile_locations,
	update_current_profile,
};
use crate::controllers::translation::{
	create_translation,
	delete_translation,
	get_translation,
	update_translation,
};
use crate::middleware::{AdminLayer, AuthLayer};

/// Get the app router
pub fn get_app_router(state: AppState) -> Router {
	let api_routes = Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest("/auth", auth_routes(&state))
		.nest("/profile", profile_routes(&state))
		.nest("/locations", location_routes(&state))
		.nest("/translations", translation_routes(&state));

	Router::new()
		.merge(api_routes)
		.layer(
			ServiceBuilder::new()
				.layer(TraceLayer::new_for_http())
				.layer(TimeoutLayer::new(Duration::from_secs(10)))
				.layer(CompressionLayer::new()),
		)
		.with_state(state)
}

/// Authentication routes
fn auth_routes(state: &AppState) -> Router<AppState> {
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
		.route("/sso/{provider}", get(sso_login))
		.route("/sso/callback", get(sso_callback))
		.route(
			"/logout",
			post(logout_profile).route_layer(AuthLayer::new(state.clone())),
		)
}

/// Profile routes
fn profile_routes(state: &AppState) -> Router<AppState> {
	let protected = Router::new()
		.route("/disable/{profile_id}", post(disable_profile))
		.route("/activate/{profile_id}", post(activate_profile))
		.route_layer(AdminLayer::new(state.clone()));

	Router::new()
		.route("/", get(get_all_profiles))
		.route("/me", get(get_current_profile).patch(update_current_profile))
		.route("/{profile_id}/locations", get(get_profile_locations))
		.merge(protected)
		.route_layer(AuthLayer::new(state.clone()))
}

/// Location routes with auth protection for write operations
fn location_routes(state: &AppState) -> Router<AppState> {
	let protected = Router::new()
		.route("/{id}/approve", post(approve_location))
		.route_layer(AdminLayer::new(state.clone()))
		.route_layer(AuthLayer::new(state.clone()));

	let authenticated = Router::new()
		.route("/", post(create_location))
		.route("/{id}", post(update_location).delete(delete_location))
		.route("/{id}/image", post(upload_location_image))
		.route_layer(AuthLayer::new(state.clone()));

	Router::new()
		.route("/", get(get_locations))
		.route("/search", get(search_locations))
		.route("/positions", get(get_location_positions))
		.route("/{id}", get(get_location))
		.merge(authenticated)
		.merge(protected)
}

/// Translation routes with auth protection
fn translation_routes(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/", post(create_translation))
		.route(
			"/{id}",
			get(get_translation)
				.delete(delete_translation)
				.post(update_translation),
		)
		.route_layer(AuthLayer::new(state.clone()))
}
