use axum::http::StatusCode;
use blokmap::controllers::auth::{
	LoginEmailRequest,
	LoginUsernameRequest,
	RegisterRequest,
};

mod common;

use blokmap::models::Profile;
use common::get_test_env;
use common::wrappers::{expect_mail_to, expect_no_mail};

#[tokio::test(flavor = "multi_thread")]
async fn register() {
	let env = get_test_env(false).await;

	let response =
		expect_mail_to(env.stub_mailbox, vec!["bob@example.com"], async || {
			env.app
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

	assert!(response.maybe_cookie("blokmap_access_token").is_none());

	assert_eq!(response.status_code(), StatusCode::CREATED);
	assert_eq!(body.username, "bob".to_string());
	assert_eq!(body.email, None);
}

#[tokio::test(flavor = "multi_thread")]
async fn register_invalid_username_start() {
	let env = get_test_env(false).await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
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

#[tokio::test(flavor = "multi_thread")]
async fn register_invalid_username_symbols() {
	let env = get_test_env(false).await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
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

#[tokio::test(flavor = "multi_thread")]
async fn register_username_too_short() {
	let env = get_test_env(false).await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "a".to_string(),
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
		"username must be between 2 and 32 characters long".to_string()
	);
}

#[tokio::test(flavor = "multi_thread")]
async fn register_username_too_long() {
	let env = get_test_env(false).await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
			.post("/auth/register")
			.json(&RegisterRequest {
				username:
					"zijne-majesteit-antonius-gregorius-albertus-III-van-brugge"
						.to_string(),
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
		"username must be between 2 and 32 characters long".to_string()
	);
}

#[tokio::test(flavor = "multi_thread")]
async fn register_password_too_short() {
	let env = get_test_env(false).await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "bob".to_string(),
				password: "123".to_string(),
				email:    "bob@example.com".to_string(),
			})
			.await
	})
	.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(
		body,
		"password must be at least 16 characters long".to_string()
	);
}

#[tokio::test(flavor = "multi_thread")]
async fn register_invalid_email() {
	let env = get_test_env(false).await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "bob".to_string(),
				password: "bobdebouwer1234!".to_string(),
				email:    "appel".to_string(),
			})
			.await
	})
	.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
	assert_eq!(body, "invalid email".to_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn register_duplicate_email() {
	let env = get_test_env(false).await;

	expect_mail_to(
		env.stub_mailbox.clone(),
		vec!["bob@example.com"],
		async || {
			env.app
				.post("/auth/register")
				.json(&RegisterRequest {
					username: "bob".to_string(),
					password: "bobdebouwer1234!".to_string(),
					email:    "bob@example.com".to_string(),
				})
				.await
		},
	)
	.await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "bob2".to_string(),
				password: "bobdebouwer1234!".to_string(),
				email:    "bob@example.com".to_string(),
			})
			.await
	})
	.await;

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::CONFLICT);
	assert_eq!(body, "email is already in use".to_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn register_duplicate_username() {
	let env = get_test_env(true).await;

	let response = expect_no_mail(env.stub_mailbox, async || {
		env.app
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "bob".to_string(),
				password: "bobdebouwer1234!".to_string(),
				email:    "bob2@example.com".to_string(),
			})
			.await
	})
	.await;

	assert!(response.maybe_cookie("blokmap_access_token").is_none());

	let body = response.text();

	assert_eq!(response.status_code(), StatusCode::CONFLICT);
	assert_eq!(body, "username is already in use".to_string());
}

#[tokio::test(flavor = "multi_thread")]
async fn confirm_email() {
	let env = get_test_env(false).await;

	expect_mail_to(env.stub_mailbox, vec!["bob@example.com"], async || {
		env.app
			.post("/auth/register")
			.json(&RegisterRequest {
				username: "bob".to_string(),
				password: "bobdebouwer1234!".to_string(),
				email:    "bob@example.com".to_string(),
			})
			.await;
	})
	.await;

	let conn = env.db_guard.create_pool().get().await.unwrap();
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

	let response = env
		.app
		.post(&format!(
			"/auth/confirm_email/{}",
			email_confirmation_token.unwrap()
		))
		.await;

	let _access_token = response.cookie("blokmap_access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let response = env.app.get("/profile/me").await;
	let body = response.json::<Profile>();

	assert_eq!(response.status_code(), StatusCode::OK);
	assert_eq!(body.username, "bob".to_string());
	assert_eq!(body.email, Some("bob@example.com".to_string()));
}

#[tokio::test(flavor = "multi_thread")]
async fn login_username() {
	let env = get_test_env(true).await;

	let response = env
		.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let _access_token = response.cookie("blokmap_access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test(flavor = "multi_thread")]
async fn login_email() {
	let env = get_test_env(true).await;

	let response = env
		.app
		.post("/auth/login/email")
		.json(&LoginEmailRequest {
			email:    "bob@example.com".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let _access_token = response.cookie("blokmap_access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test(flavor = "multi_thread")]
async fn logout() {
	let env = get_test_env(true).await;

	let response = env
		.app
		.post("/auth/login/username")
		.json(&LoginUsernameRequest {
			username: "bob".to_string(),
			password: "bobdebouwer1234!".to_string(),
		})
		.await;

	let _access_token = response.cookie("blokmap_access_token");

	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

	let response = env.app.post("/auth/logout").await;

	let access_token = response.cookie("blokmap_access_token");

	assert_eq!(access_token.max_age(), Some(time::Duration::ZERO));
	assert_eq!(response.status_code(), StatusCode::NO_CONTENT);
}
