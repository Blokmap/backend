//! # Blokmap backend library

#[macro_use]
extern crate tracing;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use deadpool_diesel::postgres::{Object, Pool};

mod config;
mod error;

pub mod controllers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod schema;

pub use config::Config;
pub use error::*;

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
