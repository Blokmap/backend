use axum::http::StatusCode;
use blokmap::controllers::auth::LoginUsernameRequest;

mod helper;
use blokmap::models::Profile;
use helper::get_test_app;

#[tokio::test]
async fn get_all_profiles() {
	let (_guard, test_server) = get_test_app(true).await;

	test_server
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let response = test_server.get("/profile").await;

	assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn get_current_profile() {
	let (_guard, test_server) = get_test_app(true).await;

	test_server
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let response = test_server.get("/profile/me").await;
	let body = response.json::<Profile>();

	assert_eq!(response.status_code(), StatusCode::OK);
	assert_eq!(body.username, "bob".to_string());
}
