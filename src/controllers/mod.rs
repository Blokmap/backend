//! Defines controller functions that correspond to individual routes

use axum::extract::State;
use axum::response::NoContent;
use common::Error;
use diesel::{RunQueryDsl, sql_query};

use crate::DbPool;

pub mod auth;
pub mod authority;
pub mod location;
pub mod notifications;
pub mod opening_time;
pub mod profile;
pub mod reservation;
pub mod review;
pub mod tag;
pub mod translation;

/// Check if the database connection and webserver are functional
pub(crate) async fn healthcheck(
	State(pool): State<DbPool>,
) -> Result<NoContent, Error> {
	let conn = pool.get().await?;

	conn.interact(|conn| sql_query("SELECT 1").execute(conn)).await??;

	Ok(NoContent)
}
