use axum::http::StatusCode;
use blokmap::schemas::tag::{CreateTagRequest, TagResponse, UpdateTagRequest};
use blokmap::schemas::translation::{
	CreateTranslationRequest,
	UpdateTranslationRequest,
};

mod common;

use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn test_get_all_tags() {
	let env = TestEnv::new().await.login("test").await;

	let response = env.app.get("/tags").await;

	assert_eq!(response.status_code(), StatusCode::OK);

	let body = response.json::<Vec<TagResponse>>();

	assert!(!body.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_tag() {
	let env = TestEnv::new().await.login_admin().await;

	let create_req = CreateTagRequest {
		name: CreateTranslationRequest {
			nl: Some("Veel Plaats".to_string()),
			en: Some("Lots of space".to_string()),
			fr: Some("Beaucoup d'espace".to_string()),
			de: Some("Viel Platz".to_string()),
		},
	};

	let response = env.app.post("/tags").json(&create_req).await;

	assert_eq!(response.status_code(), StatusCode::CREATED);

	let body = response.json::<TagResponse>();

	assert!(body.id > 0);
	assert_eq!(body.name.nl, Some("Veel Plaats".to_string()));
	assert_eq!(body.name.en, Some("Lots of space".to_string()));
	assert_eq!(body.name.fr, Some("Beaucoup d'espace".to_string()));
	assert_eq!(body.name.de, Some("Viel Platz".to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_tag_not_admin() {
	let env = TestEnv::new().await.login("test").await;

	let create_req = CreateTagRequest {
		name: CreateTranslationRequest {
			nl: Some("Veel Plaats".to_string()),
			en: Some("Lots of space".to_string()),
			fr: Some("Beaucoup d'espace".to_string()),
			de: Some("Viel Platz".to_string()),
		},
	};

	let response = env.app.post("/tags").json(&create_req).await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_update_tag() {
	let env = TestEnv::new().await.login_admin().await;

	let create_req = CreateTagRequest {
		name: CreateTranslationRequest {
			nl: Some("Gratis Koffie".to_string()),
			en: Some("Free Coffee".to_string()),
			fr: Some("Café gratuit".to_string()),
			de: Some("Kostenloser Kaffee".to_string()),
		},
	};

	let create_response = env.app.post("/tags").json(&create_req).await;
	let created = create_response.json::<TagResponse>();
	assert_eq!(create_response.status_code(), StatusCode::CREATED);

	let update_req = UpdateTagRequest {
		name: UpdateTranslationRequest {
			nl: Some("Gratis Thee".to_string()),
			en: Some("Free Tea".to_string()),
			fr: Some("Thé gratuit".to_string()),
			de: Some("Kostenloser Tee".to_string()),
		},
	};

	let update_response =
		env.app.patch(&format!("/tags/{}", created.id)).json(&update_req).await;

	assert_eq!(update_response.status_code(), StatusCode::OK);

	// Check that the updated translation reflects the changes.
	let updated = update_response.json::<TagResponse>();

	assert_eq!(updated.id, created.id);
	assert_eq!(updated.name.nl, Some("Gratis Thee".to_string()));
	assert_eq!(updated.name.en, Some("Free Tea".to_string()));
	assert_eq!(updated.name.fr, Some("Thé gratuit".to_string()));
	assert_eq!(updated.name.de, Some("Kostenloser Tee".to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_update_tag_not_admin() {
	let env = TestEnv::new().await.login_admin().await;

	let create_req = CreateTagRequest {
		name: CreateTranslationRequest {
			nl: Some("Gratis Koffie".to_string()),
			en: Some("Free Coffee".to_string()),
			fr: Some("Café gratuit".to_string()),
			de: Some("Kostenloser Kaffee".to_string()),
		},
	};

	let create_response = env.app.post("/tags").json(&create_req).await;
	let created = create_response.json::<TagResponse>();
	assert_eq!(create_response.status_code(), StatusCode::CREATED);

	let env = env.login("test").await;

	let update_req = UpdateTagRequest {
		name: UpdateTranslationRequest {
			nl: Some("Gratis Thee".to_string()),
			en: Some("Free Tea".to_string()),
			fr: Some("Thé gratuit".to_string()),
			de: Some("Kostenloser Tee".to_string()),
		},
	};

	let update_response =
		env.app.patch(&format!("/tags/{}", created.id)).json(&update_req).await;

	assert_eq!(update_response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_delete_tag() {
	let env = TestEnv::new().await.login_admin().await;

	let create_req = CreateTagRequest {
		name: CreateTranslationRequest {
			nl: Some("test".to_string()),
			en: Some("test".to_string()),
			fr: Some("test".to_string()),
			de: Some("test".to_string()),
		},
	};

	let create_response = env.app.post("/tags").json(&create_req).await;
	assert_eq!(create_response.status_code(), StatusCode::CREATED);
	let created = create_response.json::<TagResponse>();

	let delete_response =
		env.app.delete(&format!("/tags/{}", created.id)).await;

	assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_delete_tag_not_admin() {
	let env = TestEnv::new().await.login_admin().await;

	let create_req = CreateTagRequest {
		name: CreateTranslationRequest {
			nl: Some("test".to_string()),
			en: Some("test".to_string()),
			fr: Some("test".to_string()),
			de: Some("test".to_string()),
		},
	};

	let create_response = env.app.post("/tags").json(&create_req).await;
	assert_eq!(create_response.status_code(), StatusCode::CREATED);
	let created = create_response.json::<TagResponse>();

	let env = env.login("test").await;

	let delete_response =
		env.app.delete(&format!("/tags/{}", created.id)).await;

	assert_eq!(delete_response.status_code(), StatusCode::FORBIDDEN);
}
