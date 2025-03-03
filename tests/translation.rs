use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use blokmap_backend::controllers::translation::CreateTranslationResponse;
use blokmap_backend::models::NewTranslation;
use blokmap_backend::{Config, create_app};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

mod helper;
use helper::TEST_DATABASE_FIXTURE;

#[tokio::test]
async fn test_create_translation() {
	use blokmap_backend::models::Language;

	let cfg = Config::from_env();

	let test_pool_guard = (*TEST_DATABASE_FIXTURE).acquire().await;
	let test_pool = test_pool_guard.create_pool();

	let app = create_app(&cfg, test_pool);

	let response = app
		.oneshot(
			Request::builder()
				.method(Method::POST)
				.header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
				.uri("/translation")
				.body(Body::from(
					serde_json::to_string(&json!(NewTranslation {
						language: Language::En,
						key:      None,
						text:     "foo".to_string(),
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

	assert_eq!(body.translation.language, Language::En);
	assert_eq!(body.translation.text, "foo".to_string());
}

#[tokio::test]
async fn test_get_translations() {
	let cfg = Config::from_env();

	let test_pool_guard = (*TEST_DATABASE_FIXTURE).acquire().await;
	let test_pool = test_pool_guard.create_pool();

	let app = create_app(&cfg, test_pool);

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
