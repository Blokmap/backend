//! Controllers for [`Location`]s

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::error::Error;

pub async fn get_all_locations() -> Result<impl IntoResponse, Error> {
	Ok((
		StatusCode::OK,
		Json(json!({
			"status": "ok"
		})),
	))
}
