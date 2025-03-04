use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

mod helper;
use helper::get_test_app;

#[tokio::test]
async fn test_get_profiles() {
	let (_guard, app) = get_test_app().await;

	let response = app
		.oneshot(
			Request::builder().uri("/profile").body(Body::empty()).unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(response.status(), StatusCode::OK);
}
