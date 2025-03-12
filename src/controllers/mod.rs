//! Defines controller functions that correspond to individual routes

use axum::extract::State;
use axum::response::NoContent;
use diesel::{RunQueryDsl, sql_query};

use crate::DbPool;
use crate::error::Error;

pub mod auth;
pub mod location;
pub mod profile;
pub mod translation;

/// Check if the database connection and webserver are functional
pub(crate) async fn healthcheck(
	State(pool): State<DbPool>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;

	conn.interact(|conn| sql_query("SELECT 1").execute(conn)).await??;

	Ok(NoContent)
}
