use axum::http::StatusCode;
use blokmap::schemas::opening_time::OpeningTimeResponse;

mod common;

use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn test_get_location_opening_times() {
	let env = TestEnv::new().await.login("test").await;

	let location = env.get_location().await.unwrap();

	let response = env
		.app
		.get(&format!("/locations/{}/opening-times", location.location.id))
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	let body = response.json::<Vec<OpeningTimeResponse>>();

	assert!(!body.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_opening_time() {
	let env = TestEnv::new().await.login_admin().await;

	let location = env.get_location().await.unwrap();

	let create_req = serde_json::json!([{
		"day":             "2025-01-01",
		"startTime":       "08:30:00",
		"endTime":         "22:00:00",
		"seatCount":       25,
		"reservableFrom":  "2024-12-01T08:30:00",
		"reservableUntil": "2024-12-30T22:00:00",
	}]);

	let response = env
		.app
		.post(&format!("/locations/{}/opening-times", location.location.id))
		.json(&create_req)
		.await;

	assert_eq!(response.status_code(), StatusCode::CREATED);

	let body = response.json::<Vec<OpeningTimeResponse>>();
	let first = &body[0];

	assert!(first.id > 0);
	assert_eq!(first.day, "2025-01-01".parse().unwrap());
	assert_eq!(first.start_time, "08:30:00".parse().unwrap());
	assert_eq!(first.end_time, "22:00:00".parse().unwrap());
	assert_eq!(first.seat_count, Some(25));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_update_opening_time() {
	let env = TestEnv::new().await.login_admin().await;

	let location = env.get_location().await.unwrap();

	let create_request = serde_json::json!([{
		"day":             "2025-01-01",
		"startTime":       "08:30:00",
		"endTime":         "22:00:00",
		"seatCount":       25,
		"reservableFrom":  "2024-12-01T08:30:00",
		"reservableUntil": "2024-12-30T22:00:00",
	}]);

	let create_response = env
		.app
		.post(&format!("/locations/{}/opening-times", location.location.id))
		.json(&create_request)
		.await;

	assert_eq!(create_response.status_code(), StatusCode::CREATED);
	let created = create_response.json::<Vec<OpeningTimeResponse>>();
	let first = &created[0];

	let update_request = serde_json::json!({
		"day": "2025-02-02",
		"startTime": "07:30:00",
		"endTime": "23:30:00",
		"seatCount": 100,
	});

	let update_response = env
		.app
		.patch(&format!(
			"/locations/{}/opening-times/{}",
			location.location.id, first.id
		))
		.json(&update_request)
		.await;

	assert_eq!(update_response.status_code(), StatusCode::OK);
	let updated = update_response.json::<OpeningTimeResponse>();

	assert_eq!(updated.id, first.id);
	assert_eq!(updated.day, "2025-02-02".parse().unwrap());
	assert_eq!(updated.start_time, "07:30:00".parse().unwrap());
	assert_eq!(updated.end_time, "23:30:00".parse().unwrap());
	assert_eq!(updated.seat_count, Some(100));
	assert_eq!(updated.reservable_from, first.reservable_from);
	assert_eq!(updated.reservable_until, first.reservable_until);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_delete_location_time() {
	let env = TestEnv::new().await.login_admin().await;

	let location = env.get_location().await.unwrap();

	let create_request = serde_json::json!([{
		"day":             "2025-01-01",
		"startTime":       "08:30:00",
		"endTime":         "22:00:00",
		"seatCount":       25,
		"reservableFrom":  "2024-12-01T08:30:00",
		"reservableUntil": "2024-12-30T22:00:00",
	}]);

	let create_response = env
		.app
		.post(&format!("/locations/{}/opening-times", location.location.id))
		.json(&create_request)
		.await;

	assert_eq!(create_response.status_code(), StatusCode::CREATED);
	let created = create_response.json::<Vec<OpeningTimeResponse>>();

	let first = &created[0];

	let delete_response = env
		.app
		.delete(&format!(
			"/locations/{}/opening-times/{}",
			location.location.id, first.id
		))
		.await;

	assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);
}
