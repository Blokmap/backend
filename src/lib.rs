//! # Blokmap backend library

#[macro_use]
extern crate tracing;

pub mod config;
pub mod database;
pub mod routes;

pub mod controllers;
pub mod error;
pub mod models;

use axum::extract::FromRef;
pub use config::Config;
pub use database::schema;
use deadpool_diesel::postgres::{Object, Pool};

/// An entire database pool
pub type DbPool = Pool;

/// A single database connection
pub type DbConn = Object;

/// Common state of the app
#[derive(Clone)]
pub struct AppState {
	pub config:        Config,
	pub database_pool: DbPool,
}

impl FromRef<AppState> for Config {
	fn from_ref(input: &AppState) -> Self { input.config.clone() }
}

impl FromRef<AppState> for DbPool {
	fn from_ref(input: &AppState) -> Self { input.database_pool.clone() }
}
