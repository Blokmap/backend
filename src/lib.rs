//! # Blokmap backend library

#[macro_use]
extern crate tracing;

pub mod config;
pub mod routes;
pub mod schema;

pub mod controllers;
pub mod error;
pub mod models;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
pub use config::Config;
use deadpool_diesel::postgres::{Object, Pool};

/// An entire database pool
pub type DbPool = Pool;

/// A single database connection
pub type DbConn = Object;

/// Common state of the app
#[derive(Clone)]
pub struct AppState {
	pub config:         Config,
	pub database_pool:  DbPool,
	pub cookie_jar_key: Key,
}

impl FromRef<AppState> for Config {
	fn from_ref(input: &AppState) -> Self { input.config.clone() }
}

impl FromRef<AppState> for DbPool {
	fn from_ref(input: &AppState) -> Self { input.database_pool.clone() }
}

impl FromRef<AppState> for Key {
	fn from_ref(input: &AppState) -> Self { input.cookie_jar_key.clone() }
}
