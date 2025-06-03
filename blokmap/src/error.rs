//! Library-wide error types and [`From`] impls

use std::collections::HashMap;
use std::sync::LazyLock;

use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::result::DatabaseErrorKind;
use thiserror::Error;
use tokio::sync::mpsc;

/// Top level application error, can be converted into a [`Response`]
#[derive(Debug, Error)]
pub enum Error {
	/// Duplicate resource created
	#[error("{0}")]
	Duplicate(String),
	/// Request/operation forbidden
	#[error("forbidden")]
	Forbidden,
	/// Opaque internal server error
	#[error("internal server error")]
	InternalServerError,
	/// Decoding an image failed somehow
	#[error("{0}")]
	InvalidImage(String),
	/// Resource not found
	#[error("not found - {0}")]
	NotFound(String),
	/// Any error related to logging in
	#[error(transparent)]
	LoginError(#[from] LoginError),
	/// Any error related to OAuth login
	#[error(transparent)]
	OAuthError(#[from] OAuthError),
	/// Any error related to parsing multipart data
	#[error(transparent)]
	MultipartError(#[from] MultipartError),
	/// Invalid or missing token
	#[error(transparent)]
	TokenError(#[from] TokenError),
	/// Resource could not be validated
	#[error("{0}")]
	ValidationError(String),
}

/// Convert an error into a [`Result`]
impl From<Error> for Result<(), Error> {
	fn from(val: Error) -> Self { Err(val) }
}

/// Convert an error into a [`Response`]
impl IntoResponse for Error {
	fn into_response(self) -> Response {
		let message = self.to_string();

		let status = match self {
			Self::Duplicate(_) => StatusCode::CONFLICT,
			Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
			Self::Forbidden
			| Self::LoginError(_)
			| Self::OAuthError(_)
			| Self::TokenError(_) => StatusCode::FORBIDDEN,
			Self::MultipartError(_) | Self::InvalidImage(_) => {
				StatusCode::BAD_REQUEST
			},
			Self::NotFound(_) => StatusCode::NOT_FOUND,
			Self::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
		};

		(status, message).into_response()
	}
}

/// Any error related to logging in
#[derive(Debug, Error)]
pub enum LoginError {
	#[error("no profile with username '{0}' was found")]
	UnknownUsername(String),
	#[error("no profile with email '{0}' was found")]
	UnknownEmail(String),
	#[error("invalid password")]
	InvalidPassword,
	#[error("profile is still awaiting email verification")]
	PendingEmailVerification,
	#[error("profile is disabled")]
	Disabled,
}

/// Any error related to OAuth login
#[derive(Debug, Error)]
pub enum OAuthError {
	#[error("invalid CSRF token provided")]
	InvalidCSRFToken,
	#[error("missing CSRF token cookie")]
	MissingCSRFTokenCookie,
	#[error("missing email field in ID token")]
	MissingEmailField,
	#[error("missing nonce cookie")]
	MissingNonceCookie,
}

/// Any error related to a token
#[derive(Debug, Error)]
pub enum TokenError {
	#[error("email confirmation token has expired")]
	ExpiredEmailToken,
	#[error("password reset token has expired")]
	ExpiredPasswordToken,
}

/// A list of possible internal errors
///
/// API end users should never see these details
#[derive(Debug, Error)]
pub enum InternalServerError {
	/// Unknown database constraint violation
	#[error("constraint error -- {0:?}")]
	ConstraintError(String),
	/// Error executing some database operation
	#[error("database error -- {0:?}")]
	DatabaseError(diesel::result::Error),
	/// Error interacting with a database connection
	#[error("database interaction error -- {0:?}")]
	DatabaseInteractionError(deadpool_diesel::InteractError),
	/// Error handling some form of I/O
	#[error("I/O error -- {0:?}")]
	IOError(std::io::Error),
	/// Error performing some image operation
	#[error("image error -- {0:?}")]
	ImageError(image::ImageError),
	/// Error hashing some value
	#[error("hash error -- {0:?}")]
	HashError(argon2::password_hash::Error),
	/// Malformed email
	#[error("invalid email -- {0:?}")]
	InvalidEmail(lettre::address::AddressError),
	/// Mailer stopped unexpectedly
	#[error("mailer stopped -- {0:?}")]
	MailerStopped(mpsc::error::SendError<lettre::Message>),
	/// Mail queue is full
	#[error("mail queue full -- {0:?}")]
	MailQueueFull(mpsc::error::TrySendError<lettre::Message>),
	/// Generic mailer error
	#[error("mail error -- {0:?}")]
	MailError(lettre::error::Error),
	/// Error acquiring database pool connection
	#[error("database pool error -- {0:?}")]
	PoolError(deadpool_diesel::PoolError),
	/// Error executing some redis operation
	#[error("redis error -- {0:?}")]
	RedisError(redis::RedisError),
}

// Map internal server errors to application errors
impl From<InternalServerError> for Error {
	fn from(value: InternalServerError) -> Self {
		error!("internal server error -- {value}");

		Self::InternalServerError
	}
}

/// Map validation errors to application errors
impl From<validator::ValidationErrors> for Error {
	fn from(err: validator::ValidationErrors) -> Self {
		let errs = err.field_errors();
		let repr = errs
			.values()
			.map(|v| {
				v.iter()
					.map(ToString::to_string)
					.collect::<Vec<String>>()
					.join("\n")
			})
			.collect::<Vec<String>>()
			.join("\n");

		Self::ValidationError(repr)
	}
}

/// Map password hashing errors to application errors
impl From<argon2::password_hash::Error> for Error {
	fn from(err: argon2::password_hash::Error) -> Self {
		match err {
			argon2::password_hash::Error::Password => {
				LoginError::InvalidPassword.into()
			},
			_ => InternalServerError::HashError(err).into(),
		}
	}
}

/// Map database interaction errors to application errors
impl From<deadpool_diesel::InteractError> for Error {
	fn from(value: deadpool_diesel::InteractError) -> Self {
		InternalServerError::DatabaseInteractionError(value).into()
	}
}

/// Map of constraint names to column names.
static CONSTRAINT_TO_COLUMN: LazyLock<HashMap<&str, &str>> =
	LazyLock::new(|| {
		HashMap::from([
			("profile_username_key", "username"),
			("profile_email_key", "email"),
			("profile_pending_email_key", "email"),
		])
	});

/// Map database result errors to application errors.
impl From<diesel::result::Error> for Error {
	fn from(err: diesel::result::Error) -> Self {
		match &err {
			// No rows returned by query that expected at least one
			diesel::result::Error::NotFound => {
				Self::NotFound("no context provided".to_string())
			},
			// Unique constraint violation
			diesel::result::Error::DatabaseError(
				DatabaseErrorKind::UniqueViolation,
				info,
			) => {
				let constraint_name = info.constraint_name().unwrap();

				match CONSTRAINT_TO_COLUMN.get(constraint_name) {
					Some(field) => {
						Self::Duplicate(format!("{field} is already in use"))
					},
					None => InternalServerError::DatabaseError(err).into(),
				}
			},
			// Foreign key constraint violation
			diesel::result::Error::DatabaseError(
				DatabaseErrorKind::ForeignKeyViolation,
				info,
			) => Error::ValidationError(info.message().to_string()),
			_ => InternalServerError::DatabaseError(err).into(),
		}
	}
}

impl From<deadpool_diesel::PoolError> for Error {
	fn from(value: deadpool_diesel::PoolError) -> Self {
		InternalServerError::PoolError(value).into()
	}
}

impl From<lettre::address::AddressError> for Error {
	fn from(err: lettre::address::AddressError) -> Self {
		InternalServerError::InvalidEmail(err).into()
	}
}

impl From<mpsc::error::SendError<lettre::Message>> for Error {
	fn from(err: mpsc::error::SendError<lettre::Message>) -> Self {
		InternalServerError::MailerStopped(err).into()
	}
}

impl From<mpsc::error::TrySendError<lettre::Message>> for Error {
	fn from(err: mpsc::error::TrySendError<lettre::Message>) -> Self {
		InternalServerError::MailQueueFull(err).into()
	}
}

impl From<lettre::error::Error> for Error {
	fn from(err: lettre::error::Error) -> Self {
		InternalServerError::MailError(err).into()
	}
}

impl From<redis::RedisError> for Error {
	fn from(err: redis::RedisError) -> Self {
		InternalServerError::RedisError(err).into()
	}
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		InternalServerError::IOError(err).into()
	}
}

impl From<image::ImageError> for Error {
	fn from(value: image::ImageError) -> Self {
		match value {
			image::ImageError::Decoding(e) => Self::InvalidImage(e.to_string()),
			image::ImageError::IoError(e) => {
				InternalServerError::IOError(e).into()
			},
			e => InternalServerError::ImageError(e).into(),
		}
	}
}

impl From<fast_image_resize::ResizeError> for Error {
	fn from(value: fast_image_resize::ResizeError) -> Self {
		Self::InvalidImage(value.to_string())
	}
}
