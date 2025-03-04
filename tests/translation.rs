use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use blokmap::controllers::translation::{
	CreateTranslationRequest, CreateTranslationResponse,
};
use blokmap::models::Language;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod helper;
use helper::get_test_app;

#[tokio::test]
async fn test_create_translation() {
	let (_guard, app) = get_test_app().await;

	let response = app
		.oneshot(
			Request::builder()
				.method(Method::POST)
				.header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
				.uri("/translation")
				.body(Body::from(
					serde_json::to_string(&json!(CreateTranslationRequest {
						language: Language::En,
						key: None,
						text: "foo".to_string(),
					}))
					.unwrap(),
				))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::CREATED);

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: CreateTranslationResponse =
		serde_json::from_slice(&body).unwrap();

	assert_eq!(body.new_translation.language, Language::En);
	assert_eq!(body.new_translation.text, "foo".to_string());
}

#[tokio::test]
async fn test_get_translations() {
	let (_guard, app) = get_test_app().await;

	let response = app
		.oneshot(
			Request::builder()
				.uri(
					"/translation/urn:uuid:\
					 A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8",
				)
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);

	let body = response.into_body().collect().await.unwrap().to_bytes();
	let body: Value = serde_json::from_slice(&body).unwrap();
	assert_eq!(body, json!([]));
}
