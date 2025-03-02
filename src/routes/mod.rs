use axum::extract::State;
use axum::response::NoContent;
use diesel::{RunQueryDsl, sql_query};

pub mod profile;

use crate::DbPool;
use crate::error::Error;

/// Check if the database connection and webserver are functional
pub async fn healthcheck(State(pool): State<DbPool>) -> Result<NoContent, Error> {
	let conn = pool.get().await?;

	conn.interact(|conn| sql_query("SELECT 1").execute(conn)).await??;

	Ok(NoContent)
}
