use std::time::Duration;

use axum::Router;
use axum::routing::{delete, get, patch, post};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
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
use crate::controllers::authority::{
	add_authority_location,
	add_authority_member,
	create_authority,
	delete_authority_member,
	get_all_authorities,
	get_authority,
	get_authority_locations,
	get_authority_members,
	update_authority,
};
use crate::controllers::healthcheck;
use crate::controllers::institution::{
	add_institution_member,
	create_institution,
	create_institution_authority,
	delete_institution_member,
	get_all_institutions,
	get_categories,
	get_institution,
	get_institution_members,
	link_authority,
};
use crate::controllers::location::{
	add_location_member,
	approve_location,
	create_location,
	delete_location,
	delete_location_image,
	delete_location_member,
	get_location,
	get_location_members,
	get_nearest_location,
	reject_location,
	reorder_location_images,
	search_locations,
	set_location_tags,
	update_location,
	upload_location_image,
};
use crate::controllers::opening_time::{
	create_location_times,
	delete_location_time,
	get_location_times,
	update_location_time,
};
use crate::controllers::profile::{
	activate_profile,
	delete_profile_avatar,
	disable_profile,
	get_all_profiles,
	get_current_profile,
	get_profile,
	get_profile_authorities,
	get_profile_locations,
	get_profile_reservations,
	get_profile_reviews,
	get_profile_stats,
	update_current_profile,
	update_profile,
	upload_profile_avatar,
};
use crate::controllers::reservation::{
	create_reservation,
	delete_reservation,
	get_reservation_for_location,
	get_reservation_for_opening_time,
};
use crate::controllers::review::{
	create_location_review,
	get_location_reviews,
	update_location_review,
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
		.nest("/authorities", authority_routes(&state))
		.nest("/locations", location_routes(&state))
		.nest("/translations", translation_routes(&state))
		.nest("/tags", tag_routes(&state))
		.nest("/institutions", institution_routes(&state));

	Router::new()
		.merge(api_routes)
		.layer(
			ServiceBuilder::new()
				.layer(TraceLayer::new_for_http())
				.layer(TimeoutLayer::new(Duration::from_secs(10)))
				.layer(CompressionLayer::new())
				.layer(CorsLayer::permissive()),
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
	let protected = Router::new()
		.route("/", get(get_all_profiles))
		.route("/me", patch(update_current_profile))
		.route("/{profile_id}", get(get_profile).patch(update_profile))
		.route(
			"/{profile_id}/avatar",
			post(upload_profile_avatar).delete(delete_profile_avatar),
		)
		.route("/{profile_id}/block", post(disable_profile))
		.route("/{profile_id}/unblock", post(activate_profile))
		.route("/{profile_id}/authorities", get(get_profile_authorities))
		.route("/{profile_id}/locations", get(get_profile_locations))
		.route("/{profile_id}/reservations", get(get_profile_reservations))
		.route("/{profile_id}/reviews", get(get_profile_reviews))
		.route("/{profile_id}/stats", get(get_profile_stats))
		.route_layer(AuthLayer::new(state.clone()));

	Router::new().route("/me", get(get_current_profile)).merge(protected)
}

/// Location routes with auth protection for write operations
fn location_routes(state: &AppState) -> Router<AppState> {
	let protected = Router::new()
		.route("/", post(create_location))
		.route("/{id}", patch(update_location).delete(delete_location))
		.route("/{id}/approve", post(approve_location))
		.route("/{id}/reject", post(reject_location))
		.route("/{id}/tags", post(set_location_tags))
		.route(
			"/{id}/members",
			get(get_location_members).post(add_location_member),
		)
		.route("/{id}/members/{profile_id}", delete(delete_location_member))
		.route("/{id}/images", post(upload_location_image))
		.route("/{id}/images/{image_id}", delete(delete_location_image))
		.route("/{id}/images/reorder", post(reorder_location_images))
		.route(
			"/{id}/opening-times",
			get(get_location_times).post(create_location_times),
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
		.route(
			"/{id}/reviews",
			get(get_location_reviews).post(create_location_review),
		)
		.route("/{id}/reviews/{review_id}", patch(update_location_review))
		.route_layer(AuthLayer::new(state.clone()));

	Router::new()
		.route("/", get(search_locations))
		.route("/{id}", get(get_location))
		.route("/nearest", get(get_nearest_location))
		.merge(protected)
}

fn authority_routes(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/", get(get_all_authorities).post(create_authority))
		.route("/{id}", get(get_authority).patch(update_authority))
		.route(
			"/{id}/locations",
			get(get_authority_locations).post(add_authority_location),
		)
		.route(
			"/{id}/members",
			get(get_authority_members).post(add_authority_member),
		)
		.route("/{a_id}/members/{p_id}", delete(delete_authority_member))
		.route_layer(AuthLayer::new(state.clone()))
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

fn institution_routes(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/", get(get_all_institutions).post(create_institution))
		.route("/categories", get(get_categories))
		.route("/{id}", get(get_institution))
		.route("/{id}/authority", post(create_institution_authority))
		.route("/{i_id}/link/{a_id}", post(link_authority))
		.route(
			"/{id}/members",
			get(get_institution_members).post(add_institution_member),
		)
		.route("/{i_id}/members/{p_id}", delete(delete_institution_member))
		.route_layer(AuthLayer::new(state.clone()))
}
