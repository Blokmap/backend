//! Defines controller functions that correspond to individual routes

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use diesel::{RunQueryDsl, sql_query};
use serde_json::{Value, json};

use crate::DbPool;
use crate::error::Error;

pub mod profile;
pub mod translation;

/// Check if the database connection and webserver are functional
pub(crate) async fn healthcheck(
	State(pool): State<DbPool>,
) -> Result<(StatusCode, Json<Value>), Error> {
	let conn = pool.get().await?;

	conn.interact(|conn| sql_query("SELECT 1").execute(conn)).await??;

	Ok((StatusCode::OK, Json(json!({ "status": "ok" }))))
}
