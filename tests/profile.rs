use axum::body::Body;
use axum::http::{Request, StatusCode};
use blokmap_backend::{Config, create_app};
use tower::ServiceExt;

mod helper;
use helper::TEST_DATABASE_FIXTURE;

#[tokio::test]
async fn test_get_profiles() {
	let cfg = Config::from_env();

	let test_pool_guard = (*TEST_DATABASE_FIXTURE).acquire().await;
	let test_pool = test_pool_guard.create_pool();

	let app = create_app(&cfg, test_pool);

	let response = app
		.oneshot(
			Request::builder().uri("/profile").body(Body::empty()).unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
}
