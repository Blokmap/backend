use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use validator_derive::Validate;

static USERNAME_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9-_]*$").unwrap());

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
	#[validate(regex(
		path = *USERNAME_REGEX,
		message = "username must start with a letter and only contain letters, numbers, dashes, or underscores",
		code = "username-regex"
	))]
	#[validate(length(
		min = 2,
		max = 32,
		message = "username must be between 2 and 32 characters long",
		code = "username-length"
	))]
	pub username:   String,
	#[validate(length(
		min = 8,
		message = "password must be at least 8 characters long",
		code = "password-length"
	))]
	pub password:   String,
	#[validate(email(message = "invalid email", code = "email"))]
	pub email:      String,
	#[validate(length(
		min = 2,
		max = 32,
		message = "first name must be between 2 and 32 characters long",
		code = "first-name-length"
	))]
	pub first_name: String,
	#[validate(length(
		min = 2,
		max = 32,
		message = "last name must be between 2 and 32 characters long",
		code = "last-name-length"
	))]
	pub last_name:  String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PasswordResetRequest {
	pub username: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct PasswordResetData {
	pub token:    String,
	#[validate(length(
		min = 16,
		message = "password must be at least 16 characters long",
		code = "password-length"
	))]
	pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoginRequest {
	pub username: String,
	pub password: String,

	#[serde(default)]
	pub remember: bool,
}
