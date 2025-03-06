use axum::http::StatusCode;
use blokmap::controllers::auth::LoginUsernameRequest;

mod helper;
use helper::get_test_app;

#[tokio::test]
async fn test_get_profiles() {
	let (_guard, test_server) = get_test_app().await;

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
