use std::time::Duration;

use axum::Router;
use axum::routing::{delete, get, patch, post};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::controllers::auth::sso::{sso_callback, sso_login};
use crate::controllers::auth::{
	confirm_email,
	login_profile,
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
	delete_location_image,
	get_location,
	reject_location,
	search_locations,
	update_location,
	upload_location_image,
};
use crate::controllers::opening_time::{
	create_location_time,
	delete_location_time,
	get_location_times,
	update_location_time,
};
use crate::controllers::profile::{
	activate_profile,
	disable_profile,
	get_all_profiles,
	get_current_profile,
	get_profile_locations,
	get_profile_reservations,
	update_current_profile,
};
use crate::controllers::reservation::{
	create_reservation,
	delete_reservation,
	get_reservation_for_location,
	get_reservation_for_opening_time,
};
use crate::controllers::tag::{
	create_tag,
	delete_tag,
	get_all_tags,
	update_tag,
};
use crate::controllers::translation::{
	create_translation,
	delete_translation,
	get_translation,
	update_translation,
};
use crate::middleware::AuthLayer;

/// Get the app router
pub fn get_app_router(state: AppState) -> Router {
	let api_routes = Router::new()
		.route("/healthcheck", get(healthcheck))
		.nest("/auth", auth_routes(&state))
		.nest("/profiles", profile_routes(&state))
		.nest("/locations", location_routes(&state))
		.nest("/translations", translation_routes(&state))
		.nest("/tags", tag_routes(&state));

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
		.route("/login", post(login_profile))
		.route("/sso/{provider}", get(sso_login))
		.route("/sso/callback", get(sso_callback))
		.route(
			"/logout",
			post(logout_profile).route_layer(AuthLayer::new(state.clone())),
		)
}

/// Profile routes
fn profile_routes(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/", get(get_all_profiles))
		.route("/me", get(get_current_profile).patch(update_current_profile))
		.route("/{profile_id}/locations", get(get_profile_locations))
		.route("/{profile_id}/reservations", get(get_profile_reservations))
		.route("/{profile_id}/block", post(disable_profile))
		.route("/{profile_id}/unblock", post(activate_profile))
		.route_layer(AuthLayer::new(state.clone()))
}

/// Location routes with auth protection for write operations
fn location_routes(state: &AppState) -> Router<AppState> {
	let protected = Router::new()
		.route("/", post(create_location))
		.route("/{id}", patch(update_location).delete(delete_location))
		.route("/{id}/approve", post(approve_location))
		.route("/{id}/reject", post(reject_location))
		.route("/{id}/images", post(upload_location_image))
		.route("/{id}/images/{image_id}", delete(delete_location_image))
		.route(
			"/{id}/opening-times",
			get(get_location_times).post(create_location_time),
		)
		.route(
			"/{id}/opening-times/{time_id}",
			patch(update_location_time).delete(delete_location_time),
		)
		.route("/{l_id}/reservations", get(get_reservation_for_location))
		.route(
			"/{l_id}/opening-times/{t_id}/reservations",
			get(get_reservation_for_opening_time).post(create_reservation),
		)
		.route(
			"/{l_id}/opening-times/{t_id}/reservations/{r_id}",
			delete(delete_reservation),
		)
		.route_layer(AuthLayer::new(state.clone()));

	Router::new()
		.route("/", get(search_locations))
		.route("/{id}", get(get_location))
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
				.patch(update_translation),
		)
		.route_layer(AuthLayer::new(state.clone()))
}

fn tag_routes(state: &AppState) -> Router<AppState> {
	let protected = Router::new()
		.route("/", post(create_tag))
		.route("/{id}", patch(update_tag).delete(delete_tag))
		.route_layer(AuthLayer::new(state.clone()));

	Router::new().route("/", get(get_all_tags)).merge(protected)
}
