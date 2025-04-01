mod common;
use blokmap::models::{NewTranslation, Translation};
use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn create_location_test() {
	let env = TestEnv::new().await.create_and_login_test_user().await;
	let conn = env.db_guard.create_pool().get().await.unwrap();

	// Create a translation for the location name & description.
	let translation = NewTranslation {
		nl: Some("Locatie".to_string()),
		en: Some("Location".to_string()),
		de: Some("Standort".to_string()),
		fr: Some("Emplacement".to_string()),
	};

	let translation: Translation = translation.insert(&conn).await.unwrap();

	// Attempt to create a location with wrong name and description
	// translation FKs.
	let location = env
		.app
		.post("/locations")
		.json(&serde_json::json!({
			"name": "Test Location",
			"descriptionId": 69,
			"excerptId": -69,
			"seatCount": 10,
			"isReservable": true,
			"isVisible": true,
			"street": "Test Street",
			"number": "123",
			"zip": "1234AB",
			"city": "Test City",
			"province": "Test Province",
			"latitude": 52.3702,
			"longitude": 4.8952,
		}))
		.await;

	println!("Status Code: {}", location.status_code());
}

#[tokio::test(flavor = "multi_thread")]
async fn get_location_test() { todo!() }

#[tokio::test(flavor = "multi_thread")]
async fn get_locations_test() { todo!() }

#[tokio::test(flavor = "multi_thread")]
async fn get_location_positions_test() { todo!() }

#[tokio::test(flavor = "multi_thread")]
async fn update_location_test() { todo!() }

#[tokio::test(flavor = "multi_thread")]
async fn delete_location_test() { todo!() }
