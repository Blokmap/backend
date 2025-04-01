use axum::http::StatusCode;
use blokmap::controllers::auth::LoginUsernameRequest;
use blokmap::controllers::translation::{
	CreateTranslationRequest,
	CreateTranslationResponse,
};
use blokmap::models::{Language, Translation};
use serde_json::{Value, json};

mod common;

use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn create_translation() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let create_response = env
		.app
		.post("/translation")
		.json(&CreateTranslationRequest {
			language: Language::En,
			key:      None,
			text:     "foo".to_string(),
		})
		.await;

	assert_eq!(create_response.status_code(), StatusCode::CREATED);

	let create_body = create_response.json::<CreateTranslationResponse>();

	assert_eq!(create_body.new_translation.language, Language::En);
	assert_eq!(create_body.new_translation.text, "foo".to_string());

	let get_response = env
		.app
		.get(&format!(
			"/translation/{}/{:?}",
			create_body.key, create_body.new_translation.language
		))
		.await;

	assert_eq!(get_response.status_code(), StatusCode::OK);

	let get_body = get_response.json::<Translation>();

	assert_eq!(get_body.language, Language::En);
	assert_eq!(get_body.key, create_body.key);
	assert_eq!(get_body.text, "foo".to_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn get_invalid_translations() {
	let env = TestEnv::new().await;

	env.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "test".to_string(),
			password: "foo".to_string(),
		})
		.await;

	let response = env
		.app
		.get("/translation/urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8")
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	let body = response.json::<Value>();

	assert_eq!(body, json!([]));
}
