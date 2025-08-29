mod common;
use axum::http::StatusCode;
use blokmap::schemas::location::LocationResponse;
use blokmap::schemas::pagination::PaginatedResponse;
use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn create_location_test() {
	let env = TestEnv::new().await.login("test").await;

	let response = env
		.app
		.post("/locations")
		.json(&serde_json::json!({
			"name": "Test Location",
			"description": {
				"nl": "test description",
			},
			"excerpt": {
				"nl": "test excerpt",
			},
			"seatCount": 10,
			"isReservable": true,
			"maxReservationLength": 12,
			"isVisible": true,
			"street": "Test Street",
			"number": "123",
			"zip": "1234AB",
			"city": "Test City",
			"province": "Test Province",
			"country": "BE",
			"latitude": 52.0,
			"longitude": 4.0
		}))
		.await;

	assert_eq!(response.status_code(), StatusCode::CREATED);

	let location = response.json::<LocationResponse>();
	assert_eq!(location.name, "Test Location");

	let description = location.description.unwrap();
	assert!(description.nl.is_some());

	let excerpt = location.excerpt.unwrap();
	assert!(excerpt.nl.is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn get_location_test() {
	let env = TestEnv::new().await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Get the location by ID from the app router
	let response = env
		.app
		.get(format!("/locations/{}", location.primitive.id).as_str())
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);
	let location_response = response.json::<LocationResponse>();

	assert_eq!(location_response.id, location.primitive.id);
	assert_eq!(location_response.name, location.primitive.name);
}

#[tokio::test(flavor = "multi_thread")]
async fn get_locations_test() {
	let env = TestEnv::new().await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	let response = env.app.get("/locations").await;

	assert_eq!(response.status_code(), StatusCode::OK);

	// Check if the location is in the response
	let locations = response.json::<PaginatedResponse<Vec<LocationResponse>>>();
	assert!(locations.data.iter().any(|l| l.id == location.primitive.id));
	assert!(locations.data.iter().any(|l| l.name == location.primitive.name));
}

#[tokio::test(flavor = "multi_thread")]
async fn search_locations_test() {
	let env = TestEnv::new().await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Get the locations from the app router
	// Use the location above to fill the query parameters
	let response = env
		.app
		.get("/locations")
		.add_query_params([
			("northEastLat", location.primitive.latitude + 1.0),
			("northEastLng", location.primitive.longitude + 1.0),
			("southWestLat", location.primitive.latitude - 1.0),
			("southWestLng", location.primitive.longitude - 1.0),
		])
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	// Check if the location is in the response
	let locations = response.json::<PaginatedResponse<Vec<LocationResponse>>>();
	assert!(locations.data.iter().any(|l| l.id == location.primitive.id));
	assert!(locations.data.iter().any(|l| l.name == location.primitive.name));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_location_test() {
	let env = TestEnv::new().await.login("test").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Update the location with a new name
	let response = env
		.app
		.patch(format!("/locations/{}", location.primitive.id).as_str())
		.json(&serde_json::json!({
			"name": "Updated Location",
			"isVisible": !location.primitive.is_visible,
			"isReservable": !location.primitive.is_reservable,
		}))
		.await;

	assert_eq!(response.status_code(), StatusCode::OK);

	// Check if the location is updated
	let updated_location = response.json::<LocationResponse>();
	assert_eq!(updated_location.id, location.primitive.id);
	assert_eq!(updated_location.name, "Updated Location");
	assert_eq!(updated_location.is_visible, !location.primitive.is_visible);
	assert_eq!(
		updated_location.is_reservable,
		!location.primitive.is_reservable
	);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_location_unauthorized_test() {
	let env = TestEnv::new().await.login("test2").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Attempt to update the location without admin privileges
	let response = env
		.app
		.patch(format!("/locations/{}", location.primitive.id).as_str())
		.json(&serde_json::json!({
			"name": "Updated Location",
			"isVisible": !location.primitive.is_visible,
			"isReservable": !location.primitive.is_reservable,
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
		.post(format!("/locations/{}/approve", location.primitive.id).as_str())
		.await;

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	// Check if the location is approved
	let updated_location = env
		.app
		.get(&format!("/locations/{}?approved_by=true", location.primitive.id))
		.await
		.json::<LocationResponse>();

	assert_eq!(updated_location.approved_by.unwrap().unwrap().id, profile.id);
}

#[tokio::test(flavor = "multi_thread")]
async fn approve_location_unauthorized_test() {
	let env = TestEnv::new().await.login("test").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Attempt to approve the location without admin privileges
	let response = env
		.app
		.post(format!("/locations/{}/approve", location.primitive.id).as_str())
		.await;

	assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_location_test() {
	let env = TestEnv::new().await.login("test").await;

	// Get a test location in the database
	let location = env.get_location().await.unwrap();

	// Delete the location
	let response = env
		.app
		.delete(format!("/locations/{}", location.primitive.id).as_str())
		.await;
	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	// Check if the location is deleted
	let response = env
		.app
		.get(format!("/locations/{}", location.primitive.id).as_str())
		.await;

	assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}
