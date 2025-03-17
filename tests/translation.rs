use axum::http::StatusCode;
use blokmap::models::{NewTranslation, UpdateTranslation};
use blokmap::schemas::translation::{
	CreateTranslationRequest,
	TranslationResponse,
	UpdateTranslationRequest,
};

mod common;

use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn create_translation_test() {
	let env = TestEnv::new().await.create_and_login_test_user().await;

	// Create a new translation.
	let create_req = CreateTranslationRequest {
		translation: NewTranslation {
			nl: Some("hallo".to_string()),
			en: Some("hello".to_string()),
			fr: Some("bonjour".to_string()),
			de: Some("hallo".to_string()),
		},
	};

	let response = env.app.post("/translations").json(&create_req).await;

	// Ensure we get a 201 CREATED response.
	assert_eq!(response.status_code(), StatusCode::CREATED);

	let body = response.json::<TranslationResponse>();

	// Check that the returned translation has an id and expected field values.
	assert!(body.translation.id > 0);
	assert_eq!(body.translation.nl, Some("hallo".to_string()));
	assert_eq!(body.translation.en, Some("hello".to_string()));
	assert_eq!(body.translation.fr, Some("bonjour".to_string()));
	assert_eq!(body.translation.de, Some("hallo".to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_translation_test() {
	let env = TestEnv::new().await.create_and_login_test_user().await;

	// First, create a translation.
	let create_req = CreateTranslationRequest {
		translation: NewTranslation {
			nl: Some("hallo".to_string()),
			en: Some("hello".to_string()),
			fr: Some("bonjour".to_string()),
			de: Some("hallo".to_string()),
		},
	};

	let create_response = env.app.post("/translations").json(&create_req).await;

	assert_eq!(create_response.status_code(), StatusCode::CREATED);
	let created = create_response.json::<TranslationResponse>();

	// Now, retrieve the translation using its id.
	let get_response =
		env.app.get(&format!("/translations/{}", created.translation.id)).await;

	assert_eq!(get_response.status_code(), StatusCode::OK);
	let fetched = get_response.json::<TranslationResponse>();

	// Verify that the fetched translation matches the created one.
	assert_eq!(fetched.translation.id, created.translation.id);
	assert_eq!(fetched.translation.nl, created.translation.nl);
	assert_eq!(fetched.translation.en, created.translation.en);
	assert_eq!(fetched.translation.fr, created.translation.fr);
	assert_eq!(fetched.translation.de, created.translation.de);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_translation_test() {
	let env = TestEnv::new().await.create_and_login_test_user().await;

	// Create a translation.
	let create_req = CreateTranslationRequest {
		translation: NewTranslation {
			nl: Some("hallo".to_string()),
			en: Some("hello".to_string()),
			fr: Some("bonjour".to_string()),
			de: Some("hallo".to_string()),
		},
	};

	let create_response = env.app.post("/translations").json(&create_req).await;
	let created = create_response.json::<TranslationResponse>();
	assert_eq!(create_response.status_code(), StatusCode::CREATED);

	// Update the translation by changing some fields
	let update_req = UpdateTranslationRequest {
		translation: UpdateTranslation {
			nl: Some("hallo_updated".to_string()),
			en: Some("hi".to_string()),
			fr: None,
			de: Some("hallo_updated".to_string()),
		},
	};

	let update_response = env
		.app
		.post(&format!("/translations/{}", created.translation.id))
		.json(&update_req)
		.await;

	assert_eq!(update_response.status_code(), StatusCode::OK);

	// Check that the updated translation reflects the changes.
	let updated = update_response.json::<TranslationResponse>();

	assert_eq!(updated.translation.id, created.translation.id);
	assert_eq!(updated.translation.nl, Some("hallo_updated".to_string()));
	assert_eq!(updated.translation.en, Some("hi".to_string()));
	assert_eq!(updated.translation.fr, Some("bonjour".to_string()));
	assert_eq!(updated.translation.de, Some("hallo_updated".to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_translation_test() {
	let env = TestEnv::new().await.create_and_login_test_user().await;

	// Create a translation.
	let create_req = CreateTranslationRequest {
		translation: NewTranslation {
			nl: Some("hallo".to_string()),
			en: Some("hello".to_string()),
			fr: Some("bonjour".to_string()),
			de: Some("hallo".to_string()),
		},
	};

	let create_response = env.app.post("/translations").json(&create_req).await;
	assert_eq!(create_response.status_code(), StatusCode::CREATED);
	let created = create_response.json::<TranslationResponse>();

	// Delete the translation.
	let delete_response = env
		.app
		.delete(&format!("/translations/{}", created.translation.id))
		.await;

	assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

	// Ensure the translation was deleted by attempting to retrieve it.
	let get_response =
		env.app.get(&format!("/translations/{}", created.translation.id)).await;

	assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}
