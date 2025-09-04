//! Library-wide error types and [`From`] impls

use std::collections::HashMap;
use std::sync::LazyLock;

use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{NaiveDateTime, NaiveTime};
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
	/// An error that should never happen
	#[error("{0}")]
	Infallible(String),
	/// Opaque internal server error
	#[error("internal server error")]
	InternalServerError,
	/// Decoding an image failed somehow
	#[error("{0}")]
	InvalidImage(String),
	/// The request attempted to set invalid permissions
	#[error("invalid permissions")]
	InvalidRolePermissions,
	/// Resource not found
	#[error("not found - {0}")]
	NotFound(String),
	/// Any error related to logging in
	#[error(transparent)]
	LoginError(#[from] LoginError),
	/// Some data in the request was missing
	#[error("{0}")]
	MissingRequestData(String),
	/// Any error related to deserializing multipart data
	#[error(transparent)]
	MultipartSerializationError(#[from] MultipartError),
	/// Any error related to parsing multipart data
	#[error(transparent)]
	MultipartParseError(#[from] MultipartParseError),
	/// Any error related to OAuth login
	#[error(transparent)]
	OAuthError(#[from] OAuthError),
	/// Invalid pagination options
	#[error(transparent)]
	PaginationError(#[from] PaginationError),
	/// Invalid or missing token
	#[error(transparent)]
	TokenError(#[from] TokenError),
	/// Any error related to creating a reservation
	#[error(transparent)]
	CreateReservationError(#[from] CreateReservationError),
	/// Resource could not be validated
	#[error("{0}")]
	ValidationError(String),
}

impl Error {
	/// Return a unique identifying code for this error
	///
	/// An error code should never be reused once its assigned to avoid
	/// unexpectedly breaking the frontend
	fn code(&self) -> &'static str {
		match self {
			Self::Duplicate(_) => "duplicate",
			Self::Forbidden => "forbidden",
			Self::Infallible(_) => "infallible",
			Self::InternalServerError => "internal_server_error",
			Self::InvalidImage(_) => "invalid_image",
			Self::InvalidRolePermissions => "invalid_role_permissions",
			Self::NotFound(_) => "not_found",
			Self::LoginError(e) => {
				match e {
					LoginError::UnknownProfile => "unknown_profile",
					LoginError::PendingEmailVerification => {
						"pending_email_verification"
					},
					LoginError::Disabled => "disabled",
				}
			},
			Self::OAuthError(e) => {
				match e {
					OAuthError::InvalidCSRFToken => "invalid_csrf_token",
					OAuthError::MissingCSRFTokenCookie => {
						"missing_csrf_token_cookie"
					},
					OAuthError::MissingEmailField => "missing_email_field",
					OAuthError::MissingNonceCookie => "missing_nonce_cookie",
					OAuthError::UnknownProvider(_) => "unknown_provider",
				}
			},
			Self::MultipartSerializationError(_) => "multipart_serialization",
			Self::MultipartParseError(e) => {
				match e {
					MultipartParseError::MissingField { .. } => {
						"multipart_missing_field"
					},
					MultipartParseError::NamelessField => {
						"multipart_nameless_field"
					},
					MultipartParseError::UnknownField { .. } => {
						"multipart_unknown_field"
					},
					MultipartParseError::WrongFieldType { .. } => {
						"multipart_wrong_type"
					},
				}
			},
			Self::TokenError(e) => {
				match e {
					TokenError::MissingAccessToken => "missing_access_token",
					TokenError::MissingSession => "missing_session",
					TokenError::ExpiredEmailToken => "expired_email_token",
					TokenError::ExpiredPasswordToken => {
						"expired_password_token"
					},
				}
			},
			Self::CreateReservationError(e) => {
				match e {
					CreateReservationError::OutOfBounds { .. } => {
						"out_of_bounds"
					},
					CreateReservationError::NotReservableYet(_) => {
						"not_reservable_yet"
					},
					CreateReservationError::NotReservableAnymore(_) => {
						"not_reservable_anymore"
					},
					CreateReservationError::ReservationTooShort(_) => {
						"reservation_too_short"
					},
					CreateReservationError::ReservationTooLong(_) => {
						"reservation_too_long"
					},
					CreateReservationError::Full(_) => "full",
				}
			},
			Self::ValidationError(_) => "validation_error",
			Self::PaginationError(e) => {
				match e {
					PaginationError::OffsetTooLarge => "offset_too_large",
				}
			},
			Self::MissingRequestData(_) => "missing_request_data",
		}
	}

	/// Return additional information about the error
	fn info(&self) -> Option<String> {
		match self {
			Self::Duplicate(m)
			| Self::InvalidImage(m)
			| Self::NotFound(m)
			| Self::ValidationError(m) => Some(m.to_owned()),
			Self::CreateReservationError(e) => {
				match e {
					CreateReservationError::OutOfBounds { start, end } => {
						Some(
							serde_json::json!({"start": start, "end": end})
								.to_string(),
						)
					},
					CreateReservationError::NotReservableYet(from) => {
						Some(serde_json::json!({"from": from}).to_string())
					},
					CreateReservationError::NotReservableAnymore(until) => {
						Some(serde_json::json!({"until": until}).to_string())
					},
					CreateReservationError::ReservationTooShort(min) => {
						Some(serde_json::json!({"min": min}).to_string())
					},
					CreateReservationError::ReservationTooLong(max) => {
						Some(serde_json::json!({"max": max}).to_string())
					},
					CreateReservationError::Full(blocks) => {
						Some(serde_json::json!({"blocks": blocks}).to_string())
					},
				}
			},
			Self::OAuthError(OAuthError::UnknownProvider(p)) => {
				Some(serde_json::json!({"provider": p}).to_string())
			},
			Self::MultipartParseError(e) => {
				match e {
					MultipartParseError::MissingField { expected_field } => {
						Some(
							serde_json::json!({
								"expected_field": expected_field,
							})
							.to_string(),
						)
					},
					MultipartParseError::UnknownField { field_name } => {
						Some(
							serde_json::json!({
								"field_name": field_name,
							})
							.to_string(),
						)
					},
					MultipartParseError::WrongFieldType {
						field_name,
						expected_ty,
						..
					} => {
						Some(
							serde_json::json!({
								"field_name": field_name,
								"expected_type": expected_ty,
							})
							.to_string(),
						)
					},
					MultipartParseError::NamelessField => None,
				}
			},
			_ => None,
		}
	}
}

/// Convert an error into a [`Response`]
#[rustfmt::skip]
impl IntoResponse for Error {
	fn into_response(self) -> Response {
		error!("{self:?}");

		let message = self.to_string();

		let data = serde_json::json!({
			"message": message,
			"code": self.code(),
			"info": self.info(),
		});

		let status = match self {
			Self::Duplicate(_) => StatusCode::CONFLICT,
			Self::InternalServerError | Self::Infallible(_) => {
				StatusCode::INTERNAL_SERVER_ERROR
			},
			Self::TokenError(
				TokenError::MissingAccessToken | TokenError::MissingSession,
			) => StatusCode::UNAUTHORIZED,
			Self::NotFound(_)
			| Self::LoginError(LoginError::UnknownProfile) => StatusCode::NOT_FOUND,
			Self::Forbidden
			| Self::LoginError(_)
			| Self::OAuthError(OAuthError::InvalidCSRFToken)
			| Self::TokenError(_) => StatusCode::FORBIDDEN,
			Self::MultipartSerializationError(_)
			| Self::InvalidImage(_)
			| Self::CreateReservationError(_)
			| Self::PaginationError(_)
			| Self::OAuthError(
				OAuthError::MissingCSRFTokenCookie
				| OAuthError::MissingEmailField
				| OAuthError::MissingNonceCookie
				| OAuthError::UnknownProvider(_),
			) => StatusCode::BAD_REQUEST,
			Self::InvalidRolePermissions
			| Self::ValidationError(_)
			| Self::MissingRequestData(_)
			| Self::MultipartParseError(_) => {
				StatusCode::UNPROCESSABLE_ENTITY
			},
		};

		(status, axum::Json(data)).into_response()
	}
}

/// Any error related to logging in
#[derive(Debug, Error)]
pub enum LoginError {
	#[error("no profile with these login details was found")]
	UnknownProfile,
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
	#[error("unknown OAuth provider: {0:?}")]
	UnknownProvider(String),
}

/// Any error related to a token
#[derive(Debug, Error)]
pub enum TokenError {
	#[error("missing or invalid access token")]
	MissingAccessToken,
	#[error("missing session")]
	MissingSession,

	#[error("email confirmation token has expired")]
	ExpiredEmailToken,
	#[error("password reset token has expired")]
	ExpiredPasswordToken,
}

#[derive(Debug, Error)]
pub enum MultipartParseError {
	#[error(
		"incorrect field type for '{field_name}', expected '{expected_ty}'"
	)]
	WrongFieldType { field_name: String, expected_ty: String },
	#[error("missing field '{expected_field}'")]
	MissingField { expected_field: String },
	#[error("unknown field '{field_name}'")]
	UnknownField { field_name: String },
	#[error("nameless field")]
	NamelessField,
}

#[derive(Debug, Error)]
pub enum CreateReservationError {
	/// The request was out of bounds for the given opening time
	#[error("reservation out of bounds for the opening time")]
	OutOfBounds { start: NaiveTime, end: NaiveTime },
	/// The request was made before the timeslot was reservable
	#[error("this timeslot is not reservable yet")]
	NotReservableYet(NaiveDateTime),
	/// The request was made after the timeslot was reservable
	#[error("this timeslot is not reservable anymore")]
	NotReservableAnymore(NaiveDateTime),
	/// The amount of blocks reserved was less than the minimum reservation
	/// length
	#[error("the reserved amount of time was too short")]
	ReservationTooShort(i32),
	/// The amount of blocks reserved was more than the maximum reservation
	/// length
	#[error("the reserved amount of time was too long")]
	ReservationTooLong(i32),
	/// The reservation exceeds the capacity of the opening time at one or more
	/// blocks
	#[error("the reservation would overoccupy some blocks")]
	Full(Vec<i32>),
}

#[derive(Debug, Error)]
pub enum PaginationError {
	#[error("the offset is too large for the amount of data")]
	OffsetTooLarge,
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
	ImageError(image_processing::ImageError),
	/// Error joining some async task
	#[error("join error -- {0:?}")]
	JoinError(tokio::task::JoinError),
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
	/// Error related to `serde_json`
	#[error("serde_json error -- {0:?}")]
	SerdeJsonError(serde_json::Error),
	/// Attempted to extract a session from a request that has not been
	/// authorized
	#[error("attempted to extract session without checking authorization")]
	SessionWithoutAuthError,
	/// Failed to parse a url
	#[error("could not parse url -- {0:?}")]
	UrlParseError(url::ParseError),
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
				LoginError::UnknownProfile.into()
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

impl From<serde_json::Error> for Error {
	fn from(err: serde_json::Error) -> Self {
		InternalServerError::SerdeJsonError(err).into()
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

impl From<image_processing::ImageError> for Error {
	fn from(value: image_processing::ImageError) -> Self {
		match value {
			image_processing::ImageError::Decoding(e) => {
				Self::InvalidImage(e.to_string())
			},
			image_processing::ImageError::Unsupported(e) => {
				Self::InvalidImage(e.to_string())
			},
			image_processing::ImageError::IoError(e) => {
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

impl From<url::ParseError> for Error {
	fn from(err: url::ParseError) -> Self {
		InternalServerError::UrlParseError(err).into()
	}
}
