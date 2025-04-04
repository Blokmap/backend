use axum::http::StatusCode;
use blokmap::controllers::auth::LoginUsernameRequest;
use blokmap::models::{Profile, ProfileState, ProfileUpdate};

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

#[tokio::test(flavor = "multi_thread")]
async fn disable_profile() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test-admin".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let response = env.app.get("/profile").await;
	let profiles: Vec<Profile> = response.json();
	let test_id =
		profiles.iter().find(|p| p.username == "test").map(|p| p.id).unwrap();

	let response = env.app.post(&format!("/profile/disable/{test_id}")).await;

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Disabled);
}

#[tokio::test(flavor = "multi_thread")]
async fn disable_profile_not_admin() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let response = env.app.get("/profile").await;
	let profiles: Vec<Profile> = response.json();
	let test_id =
		profiles.iter().find(|p| p.username == "test").map(|p| p.id).unwrap();

	let response = env.app.post(&format!("/profile/disable/{test_id}")).await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Active);
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_profile() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test-admin".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let test = Profile::get_all(&conn)
		.await
		.unwrap()
		.into_iter()
		.find(|p| p.username == "test-disabled")
		.unwrap();
	let test_id = test.id;

	let response = env.app.post(&format!("/profile/activate/{test_id}")).await;

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Active);
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_profile_not_admin() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let test = Profile::get_all(&conn)
		.await
		.unwrap()
		.into_iter()
		.find(|p| p.username == "test-disabled")
		.unwrap();
	let test_id = test.id;

	let response = env.app.post(&format!("/profile/activate/{test_id}")).await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(test_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Disabled);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_current_profile_username() {
	let env = TestEnv::new().await.create_test_user().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
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
	let env = TestEnv::new().await.create_test_user().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let conn = env.db_guard.create_pool().get().await.unwrap();
	let old_profile: Profile = conn
		.interact(|conn| {
			use blokmap::schema::profile::dsl::*;
			use diesel::prelude::*;

			profile.filter(username.eq("bob")).get_result(conn)
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

			profile.filter(username.eq("bob")).get_result(conn)
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
	let env = TestEnv::new()
		.await
		.create_test_user()
		.await
		.create_test_admin_user()
		.await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "alice".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let response = env.app.get("/profile").await;
	let profiles: Vec<Profile> = response.json();
	let bob_id =
		profiles.iter().find(|p| p.username == "bob").map(|p| p.id).unwrap();

	let response = env.app.post(&format!("/profile/disable/{bob_id}")).await;

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(bob_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Disabled);
}

#[tokio::test(flavor = "multi_thread")]
async fn disable_profile_not_admin() {
	let env = TestEnv::new().await.create_test_user().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let response = env.app.get("/profile").await;
	let profiles: Vec<Profile> = response.json();
	let bob_id =
		profiles.iter().find(|p| p.username == "bob").map(|p| p.id).unwrap();

	let response = env.app.post(&format!("/profile/disable/{bob_id}")).await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(bob_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Active);
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_profile() {
	let env = TestEnv::new()
		.await
		.create_test_user()
		.await
		.create_test_admin_user()
		.await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "alice".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let mut bob = Profile::get_all(&conn)
		.await
		.unwrap()
		.into_iter()
		.find(|p| p.username == "bob")
		.unwrap();
	let bob_id = bob.id;
	bob.state = ProfileState::Disabled;
	bob.update(&conn).await.unwrap();

	let response = env.app.post(&format!("/profile/activate/{bob_id}")).await;

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let bob = Profile::get(bob_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Active);
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_profile_not_admin() {
	let env = TestEnv::new().await.create_test_user().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let mut bob = Profile::get_all(&conn)
		.await
		.unwrap()
		.into_iter()
		.find(|p| p.username == "bob")
		.unwrap();
	let bob_id = bob.id;
	bob.state = ProfileState::Disabled;
	bob.update(&conn).await.unwrap();

	let response = env.app.post(&format!("/profile/activate/{bob_id}")).await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

	let pool = env.db_guard.create_pool();
	let conn = pool.get().await.unwrap();
	let bob = Profile::get(bob_id, &conn).await.unwrap();

	assert_eq!(bob.state, ProfileState::Disabled);
}
