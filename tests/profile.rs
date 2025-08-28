use axum::http::StatusCode;
use blokmap::schemas::auth::LoginRequest;
use blokmap::schemas::pagination::{PaginatedResponse, PaginationOptions};
use blokmap::schemas::reservation::ReservationResponse;
use db::ProfileState;
use primitives::PrimitiveProfile;
use profile::Profile;

mod common;

use blokmap::schemas::location::LocationResponse;
use blokmap::schemas::profile::{ProfileResponse, UpdateProfileRequest};
use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn get_all_profiles() {
	let env = TestEnv::new().await;

	let response = env
		.app
		.post("/auth/login")
		.json(&LoginRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
			remember: false,
		})
		.await;

	let _access_token = response.cookie("blokmap_access_token");

	let response = env.app.get("/profiles").await;

	assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_current_profile() {
	let env = TestEnv::new().await.login("test").await;

	let response = env.app.get("/profiles/me").await;
	let body = response.json::<ProfileResponse>();

	assert_eq!(response.status_code(), StatusCode::OK);
	assert_eq!(body.username, "test".to_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn update_current_profile_username() {
	let env = TestEnv::new().await.login_admin().await;

	let response = env.app.get("/profiles/me").await;
	let old_profile = response.json::<ProfileResponse>();

	let response = env
		.expect_no_mail(async || {
			env.app
				.patch("/profiles/me")
				.json(&UpdateProfileRequest {
					username:      Some("bobble".to_string()),
					first_name:    None,
					last_name:     None,
					pending_email: None,
				})
				.await
		})
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	let response = env.app.get("/profiles/me").await;
	let new_profile = response.json::<ProfileResponse>();

	assert_ne!(old_profile.username, new_profile.username);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_current_profile_pending_email() {
	let env = TestEnv::new().await.login("test").await;

	let conn = env.db_guard.create_pool().get().await.unwrap();
	let old_profile: PrimitiveProfile = conn
		.interact(|conn| {
			use db::profile::dsl::*;
			use diesel::prelude::*;

			profile.filter(username.eq("test")).get_result(conn)
		})
		.await
		.unwrap()
		.unwrap();

	let response = env
		.expect_mail_to(&["bobble@example.com"], async || {
			env.app
				.patch("/profiles/me")
				.json(&UpdateProfileRequest {
					username:      None,
					first_name:    None,
					last_name:     None,
					pending_email: Some("bobble@example.com".to_string()),
				})
				.await
		})
		.await;
	assert_eq!(response.status_code(), StatusCode::OK);

	let new_profile: PrimitiveProfile = conn
		.interact(|conn| {
			use db::profile::dsl::*;
			use diesel::prelude::*;

			profile.filter(username.eq("test")).get_result(conn)
		})
		.await
		.unwrap()
		.unwrap();

	assert_ne!(old_profile.pending_email, new_profile.pending_email);
	assert!(new_profile.email_confirmation_token.is_some());
	assert!(new_profile.email_confirmation_token_expiry.is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn disable_profile() {
	let env = TestEnv::new().await.login_admin().await;

	let response = env.app.get("/profiles").await;
	let profiles: PaginatedResponse<Vec<ProfileResponse>> = response.json();
	let test_id = profiles
		.data
		.iter()
		.find(|p| p.username == "test")
		.map(|p| p.id)
		.unwrap();

	let response = env.app.post(&format!("/profiles/{test_id}/block")).await;

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.primitive.state, ProfileState::Disabled);
}

#[tokio::test(flavor = "multi_thread")]
async fn disable_profile_not_admin() {
	let env = TestEnv::new().await.login("test").await;

	let response = env.app.get("/profiles").await;
	let profiles: PaginatedResponse<Vec<ProfileResponse>> = response.json();

	let test_id = profiles
		.data
		.iter()
		.find(|p| p.username == "test")
		.map(|p| p.id)
		.unwrap();

	let response = env.app.post(&format!("/profiles/{test_id}/block")).await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.primitive.state, ProfileState::Active);
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_profile() {
	let env = TestEnv::new().await.login_admin().await;

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let pagination = PaginationOptions::default();
	let test = Profile::get_all(pagination.into(), &conn)
		.await
		.unwrap()
		.2
		.into_iter()
		.find(|p| p.primitive.username == "test-disabled")
		.unwrap();

	let test_id = test.primitive.id;

	let response = env.app.post(&format!("/profiles/{test_id}/unblock")).await;

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.primitive.state, ProfileState::Active);
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_profile_not_admin() {
	let env = TestEnv::new().await.login("test").await;

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let pagination = PaginationOptions::default();
	let test = Profile::get_all(pagination.into(), &conn)
		.await
		.unwrap()
		.2
		.into_iter()
		.find(|p| p.primitive.username == "test-disabled")
		.unwrap();

	let test_id = test.primitive.id;

	let response = env.app.post(&format!("/profiles/{test_id}/unblock")).await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.primitive.state, ProfileState::Disabled);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_profile_locations() {
	let env = TestEnv::new().await.login("test").await;

	let response = env.app.get("/profiles/1/locations").await;
	let _ = response.json::<Vec<LocationResponse>>();

	assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_profile_reservations() {
	let env = TestEnv::new().await.login("test").await;

	let response = env.app.get("/profiles/1/reservations").await;
	let _ = response.json::<Vec<ReservationResponse>>();

	assert_eq!(response.status_code(), StatusCode::OK);
}
