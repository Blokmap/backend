/// TODO:
///   - `get_reservations_for_location`
///       - check permissions if not authenticated
///       - check permissions if not a manager
///   - `get_reservations_for_opening_time`
///       - check permissions if not authenticated
///       - check permissions if not a manager
///   - `create_reservation`
///       - check permissions if not authenticated
///       - check for out of bounds
///       - check for reservable timeframe exceeded
///       - check for reservation length
///       - check for occupation exceeded
///   - `delete_reservation`
///       - check permissions if not authenticated
///       - check permissions if not a manager
use axum::http::StatusCode;

mod common;

use blokmap::schemas::reservation::ReservationResponse;
use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn get_reservations_for_location() {
	let env = TestEnv::new().await.login("test").await;

	let location = env.get_location().await.unwrap();

	let response = env
		.app
		.get(&format!("/locations/{}/reservations", location.location.id))
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	let body = response.json::<Vec<ReservationResponse>>();

	assert!(!body.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn get_reservations_for_opening_time() {
	let env = TestEnv::new().await.login("test").await;

	let location = env.get_location().await.unwrap();
	let time = env.get_opening_time().await.unwrap();

	let response = env
		.app
		.get(&format!(
			"/locations/{}/opening-times/{}/reservations",
			location.location.id, time.opening_time.id
		))
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	let body = response.json::<Vec<ReservationResponse>>();

	assert!(!body.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn create_reservation() {
	let env = TestEnv::new().await.login("test").await;

	let location = env.get_location().await.unwrap();
	let time = env.get_opening_time().await.unwrap();

	let create_req = serde_json::json!({
		"startTime": "10:30:00",
		"endTime": "13:30:00",
	});

	let response = env
		.app
		.post(&format!(
			"/locations/{}/opening-times/{}/reservations",
			location.location.id, time.opening_time.id
		))
		.json(&create_req)
		.await;

	assert_eq!(response.status_code(), StatusCode::CREATED);

	let body = response.json::<ReservationResponse>();

	assert!(body.id > 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_reservation() {
	let env = TestEnv::new().await.login_admin().await;

	let location = env.get_location().await.unwrap();
	let time = env.get_opening_time().await.unwrap();

	let create_req = serde_json::json!({
		"startTime": "10:30:00",
		"endTime": "13:30:00",
	});

	let response = env
		.app
		.post(&format!(
			"/locations/{}/opening-times/{}/reservations",
			location.location.id, time.opening_time.id
		))
		.json(&create_req)
		.await;

	assert_eq!(response.status_code(), StatusCode::CREATED);
	let created = response.json::<ReservationResponse>();

	let delete_response = env
		.app
		.delete(&format!(
			"/locations/{}/opening-times/{}/reservations/{}",
			location.location.id, time.opening_time.id, created.id,
		))
		.await;

	assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);
}
