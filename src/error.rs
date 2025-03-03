use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("{0}")]
	Duplicate(String),
	#[error("internal server error")]
	InternalServerError,
	#[error("not found")]
	NotFound,
	#[error("{0}")]
	ValidationError(String),
}

impl From<InternalServerError> for Error {
	fn from(value: InternalServerError) -> Self {
		error!("internal server error -- {value}");

		Self::InternalServerError
	}
}

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

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		let message = self.to_string();

		let status = match self {
			Self::Duplicate(_) => StatusCode::CONFLICT,
			Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
			Self::NotFound => StatusCode::NOT_FOUND,
			Self::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
		};

		(status, message).into_response()
	}
}

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
	/// Error acquiring database pool connection
	#[error("database pool error -- {0:?}")]
	PoolError(deadpool_diesel::PoolError),
}

impl From<deadpool_diesel::InteractError> for Error {
	fn from(value: deadpool_diesel::InteractError) -> Self {
		InternalServerError::DatabaseInteractionError(value).into()
	}
}

impl From<diesel::result::Error> for Error {
	fn from(err: diesel::result::Error) -> Self {
		match &err {
			diesel::result::Error::NotFound => Self::NotFound,
			diesel::result::Error::DatabaseError(
				diesel::result::DatabaseErrorKind::UniqueViolation,
				info,
			) => {
				// Unwrap is safe as constraint_name is guaranteed to exist for
				// postgres
				let constraint_name = info.constraint_name().unwrap();

				// Standard constaint names in postgres are
				// {tablename}_{columnname}_{suffix}
				let Some(field) = constraint_name.split('_').nth(1) else {
					return InternalServerError::ConstraintError(
						constraint_name.to_string(),
					)
					.into();
				};

				Self::Duplicate(format!("'{field}' is already in use"))
			},
			_ => InternalServerError::DatabaseError(err).into(),
		}
	}
}

impl From<deadpool_diesel::PoolError> for Error {
	fn from(value: deadpool_diesel::PoolError) -> Self {
		InternalServerError::PoolError(value).into()
	}
}
