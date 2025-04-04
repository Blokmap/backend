mod common;
use axum::http::StatusCode;
use blokmap::schemas::location::LocationResponse;
use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn create_location_test() {
	let env = TestEnv::new().await.login("test").await;

	// Attempt to create a location with wrong name and description
	// translation FKs (coming from the seeder).
	let response = env
		.app
		.post("/locations")
		.json(&serde_json::json!({
			"name": "Test Location",
			"descriptionId": 1,
			"excerptId": 1,
			"seatCount": 10,
			"isReservable": true,
			"isVisible": true,
			"street": "Test Street",
			"number": "123",
			"zip": "1234AB",
			"city": "Test City",
			"province": "Test Province",
			"latitude": 52.0,
			"longitude": 4.0
		}))
		.await;

	assert_eq!(response.status_code(), StatusCode::CREATED);

	let location = response.json::<LocationResponse>();
	assert_eq!(location.name, "Test Location");

	let description = location.description.unwrap();
	assert!(description.translation.nl.is_some());

	let excerpt = location.excerpt.unwrap();
	assert!(excerpt.translation.nl.is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_location_invalid_translation() {
	let env = TestEnv::new().await.login("test").await;

	// Attempt to create a location with wrong name and description
	// translation FKs.
	let response = env
		.app
		.post("/locations")
		.json(&serde_json::json!({
			"name": "Test Location",
			"descriptionId": 69,
			"excerptId": 420,
			"seatCount": 10,
			"isReservable": true,
			"isVisible": true,
			"street": "Test Street",
			"number": "123",
			"zip": "1234AB",
			"city": "Test City",
			"province": "Test Province",
			"latitude": 52.0,
			"longitude": 4.0
		}))
		.await;

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_location_test() {
	let env = TestEnv::new().await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Get the location by ID from the app router
	let response =
		env.app.get(format!("/locations/{}", location.id).as_str()).await;

	assert_eq!(response.status_code(), StatusCode::OK);
	let location_response = response.json::<LocationResponse>();

	assert_eq!(location_response.id, location.id);
	assert_eq!(location_response.name, location.name);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_locations_test() {
	let env = TestEnv::new().await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Get the locations from the app router
	// Use the location above to fill the query parameters
	let response = env
		.app
		.get("/locations")
		.add_query_params([
			("northEastLat", location.latitude + 1.0),
			("northEastLng", location.longitude + 1.0),
			("southWestLat", location.latitude - 1.0),
			("southWestLng", location.longitude - 1.0),
		])
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	// Check if the location is in the response
	let locations = response.json::<Vec<LocationResponse>>();
	assert!(locations.iter().any(|l| l.id == location.id));
	assert!(locations.iter().any(|l| l.name == location.name));
}

#[tokio::test(flavor = "multi_thread")]
async fn get_location_positions_test() {
	let env = TestEnv::new().await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Get the location positions from the app router
	let response = env.app.get("/locations/positions").await;

	assert_eq!(response.status_code(), StatusCode::OK);

	// Check if the location is in the response
	let locations = response.json::<Vec<(f64, f64)>>();
	assert!(
		locations
			.iter()
			.any(|l| l.0 == location.latitude && l.1 == location.longitude)
	);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_location_test() {
	let env = TestEnv::new().await.login("test").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Update the location with a new name
	let response = env
		.app
		.post(format!("/locations/{}", location.id).as_str())
		.json(&serde_json::json!({
			"name": "Updated Location",
			"isVisible": !location.is_visible,
			"isReservable": !location.is_reservable,
		}))
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	// Check if the location is updated
	let updated_location = response.json::<LocationResponse>();
	assert_eq!(updated_location.id, location.id);
	assert_eq!(updated_location.name, "Updated Location");
	assert_eq!(updated_location.is_visible, !location.is_visible);
	assert_eq!(updated_location.is_reservable, !location.is_reservable);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_location_unauthorized_test() {
	let env = TestEnv::new().await.login("test2").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Attempt to update the location without admin privileges
	let response = env
		.app
		.post(format!("/locations/{}", location.id).as_str())
		.json(&serde_json::json!({
			"name": "Updated Location",
			"isVisible": !location.is_visible,
			"isReservable": !location.is_reservable,
		}))
		.await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn approve_location_test() {
	let env = TestEnv::new().await.login_admin().await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();
	let profile = env.get_admin_profile().await.unwrap();

	// Approve the location
	let response = env
		.app
		.post(format!("/locations/{}/approve", location.id).as_str())
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	// Check if the location is approved
	let updated_location = env.get_location().await.unwrap();
	assert_eq!(updated_location.approved_by_id, Some(profile.id));
}

#[tokio::test(flavor = "multi_thread")]
async fn approve_location_unauthorized_test() {
	let env = TestEnv::new().await.login("test").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Attempt to approve the location without admin privileges
	let response = env
		.app
		.post(format!("/locations/{}/approve", location.id).as_str())
		.await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_location_test() {
	let env = TestEnv::new().await.login("test").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Delete the location
	let response =
		env.app.delete(format!("/locations/{}", location.id).as_str()).await;
	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	// Check if the location is deleted
	let response =
		env.app.get(format!("/locations/{}", location.id).as_str()).await;

	assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}
