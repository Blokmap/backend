use axum::http::StatusCode;
use blokmap::controllers::auth::{
	LoginEmailRequest,
	LoginUsernameRequest,
	RegisterRequest,
};

mod common;

use blokmap::models::Profile;
use common::get_test_app;
use common::wrappers::{expect_mail, expect_no_mail};

#[tokio::test(flavor = "multi_thread")]
async fn register() {
	let (_guard, mailbox, test_server) = get_test_app(false).await;

	let response = expect_mail(mailbox, async || {
		test_server
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "bob".to_string(),
				password: "bobdebouwer1234!".to_string(),
				email:    "bob@example.com".to_string(),
			})
			.await
	})
	.await;

	let body = response.json::<Profile>();

	assert_eq!(response.status_code(), StatusCode::CREATED);
	assert_eq!(body.username, "bob".to_string());
	assert_eq!(body.email, None);
}

#[tokio::test(flavor = "multi_thread")]
async fn register_invalid_username_start() {
	let (_guard, mailbox, test_server) = get_test_app(false).await;

	let response = expect_no_mail(mailbox, async || {
		test_server
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "123".to_string(),
				password: "bobdebouwer1234!".to_string(),
				email:    "bob@example.com".to_string(),
			})
			.await
	})
	.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(
		body,
		"username must start with a letter and only contain letters, numbers, \
		 dashes, or underscores"
			.to_string()
	);
}

#[tokio::test]
async fn register_invalid_username_symbols() {
	let (_guard, mailbox, test_server) = get_test_app(false).await;

	let response = expect_no_mail(mailbox, async || {
		test_server
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "abc.".to_string(),
				password: "bobdebouwer1234!".to_string(),
				email:    "bob@example.com".to_string(),
			})
			.await
	})
	.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(
		body,
		"username must start with a letter and only contain letters, numbers, \
		 dashes, or underscores"
			.to_string()
	);
}

#[tokio::test]
async fn register_username_too_short() {
	let (_guard, _mailbox, test_server) = get_test_app(false).await;

	let response = test_server
		.post("/auth/register")
		.json(&RegisterRequest {
			username: "a".to_string(),
			password: "bobdebouwer1234!".to_string(),
			email:    "bob@example.com".to_string(),
		})
		.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(
		body,
		"username must be between 2 and 32 characters long".to_string()
	);
}

#[tokio::test]
async fn register_username_too_long() {
	let (_guard, _mailbox, test_server) = get_test_app(false).await;

	let response = test_server
		.post("/auth/register")
		.json(&RegisterRequest {
			username:
				"zijne-majesteit-antonius-gregorius-albertus-III-van-brugge"
					.to_string(),
			password: "bobdebouwer1234!".to_string(),
			email:    "bob@example.com".to_string(),
		})
		.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(
		body,
		"username must be between 2 and 32 characters long".to_string()
	);
}

#[tokio::test]
async fn register_password_too_short() {
	let (_guard, _mailbox, test_server) = get_test_app(false).await;

	let response = test_server
		.post("/auth/register")
		.json(&RegisterRequest {
			username: "bob".to_string(),
			password: "123".to_string(),
			email:    "bob@example.com".to_string(),
		})
		.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(
		body,
		"password must be at least 16 characters long".to_string()
	);
}

#[tokio::test]
async fn register_invalid_email() {
	let (_guard, _mailbox, test_server) = get_test_app(false).await;

	let response = test_server
		.post("/auth/register")
		.json(&RegisterRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
			email:    "appel".to_string(),
		})
		.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(body, "invalid email".to_string());
}

#[tokio::test]
async fn confirm_email() {
	let (guard, _mailbox, test_server) = get_test_app(false).await;

	test_server
		.post("/auth/register")
		.json(&RegisterRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
			email:    "bob@example.com".to_string(),
		})
		.await;

	let conn = guard.create_pool().get().await.unwrap();
	let email_confirmation_token: Option<String> = conn
		.interact(|conn| {
			use blokmap::schema::profile::dsl::*;
			use diesel::prelude::*;

			profile
				.select(email_confirmation_token)
				.filter(username.eq("bob"))
				.get_result(conn)
		})
		.await
		.unwrap()
		.unwrap();

	assert!(email_confirmation_token.is_some());

	let response = test_server
		.post(&format!(
			"/auth/confirm_email/{}",
			email_confirmation_token.unwrap()
		))
		.await;

	let _access_token = response.cookie("access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let response = test_server.get("/profile/me").await;
	let body = response.json::<Profile>();

	assert_eq!(response.status_code(), StatusCode::OK);
	assert_eq!(body.username, "bob".to_string());
	assert_eq!(body.email, Some("bob@example.com".to_string()));
}

#[tokio::test]
async fn login_username() {
	let (_guard, _mailbox, test_server) = get_test_app(true).await;

	let response = test_server
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let _access_token = response.cookie("access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn login_email() {
	let (_guard, _mailbox, test_server) = get_test_app(true).await;

	let response = test_server
		.post("/auth/login/email")
		.json(&LoginEmailRequest {
			email:    "bob@example.com".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let _access_token = response.cookie("access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn logout() {
	let (_guard, _mailbox, test_server) = get_test_app(true).await;

	let response = test_server
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let _access_token = response.cookie("access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let response = test_server.post("/auth/logout").await;

	let access_token = response.cookie("access_token");

	assert_eq!(access_token.max_age(), Some(time::Duration::ZERO));
	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}
