use axum::http::StatusCode;
use blokmap::controllers::auth::LoginUsernameRequest;
use blokmap::models::{Profile, ProfileUpdate};

mod common;

use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn get_all_profiles() {
	let env = TestEnv::new().await;

	let response = env
		.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let _access_token = response.cookie("blokmap_access_token");

	let response = env.app.get("/profile").await;

	assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_current_profile() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let response = env.app.get("/profile/me").await;
	let body = response.json::<Profile>();

	assert_eq!(response.status_code(), StatusCode::OK);
	assert_eq!(body.username, "test".to_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn update_current_profile_username() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let response = env.app.get("/profile/me").await;
	let old_profile = response.json::<Profile>();

	let response = env
		.expect_no_mail(async || {
			env.app
				.patch("/profile/me")
				.json(&ProfileUpdate {
					username:      Some("bobble".to_string()),
					pending_email: None,
				})
				.await
		})
		.await;
	assert_eq!(response.status_code(), StatusCode::OK);

	let response = env.app.get("/profile/me").await;
	let new_profile = response.json::<Profile>();

	assert_ne!(old_profile.username, new_profile.username);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_current_profile_pending_email() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let conn = env.db_guard.create_pool().get().await.unwrap();
	let old_profile: Profile = conn
		.interact(|conn| {
			use blokmap::schema::profile::dsl::*;
			use diesel::prelude::*;

			profile.filter(username.eq("test")).get_result(conn)
		})
		.await
		.unwrap()
		.unwrap();

	let response = env
		.expect_mail_to(&["bobble@example.com"], async || {
			env.app
				.patch("/profile/me")
				.json(&ProfileUpdate {
					username:      None,
					pending_email: Some("bobble@example.com".to_string()),
				})
				.await
		})
		.await;
	assert_eq!(response.status_code(), StatusCode::OK);

	let new_profile: Profile = conn
		.interact(|conn| {
			use blokmap::schema::profile::dsl::*;
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
