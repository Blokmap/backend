mod common;
use blokmap::{models::{NewTranslation, Translation}, schemas::location::CreateLocationRequest};
use common::TestEnv;

#[tokio::test(flavor = "multi_thread")]
async fn create_location_test() {
	let env = TestEnv::new().await.create_and_login_test_user().await;
    let conn = env.

    let translation = NewTranslation {

    }
    translation.insert(env.con);

    let location_req = CreateLocationRequest {

    }
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
