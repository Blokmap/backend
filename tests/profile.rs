use axum::http::StatusCode;
use blokmap::controllers::auth::LoginUsernameRequest;
use blokmap::models::Profile;

mod common;

use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn get_all_profiles() {
	let env = TestEnv::new().await.create_test_user().await;

	let response = env
		.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let _access_token = response.cookie("blokmap_access_token");

	let response = env.app.get("/profile").await;

	assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_current_profile() {
	let env = TestEnv::new().await.create_test_user().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let response = env.app.get("/profile/me").await;
	let body = response.json::<Profile>();

	assert_eq!(response.status_code(), StatusCode::OK);
	assert_eq!(body.username, "bob".to_string());
}
