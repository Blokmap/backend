use axum::http::StatusCode;
use blokmap::controllers::auth::LoginUsernameRequest;

mod common;

use blokmap::models::Profile;
use common::get_test_env;

#[tokio::test(flavor = "multi_thread")]
async fn get_all_profiles() {
	let env = get_test_env(true).await;

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
	let env = get_test_env(true).await;

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
