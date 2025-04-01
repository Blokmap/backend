//! # Blokmap backend library

#[macro_use]
extern crate tracing;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use deadpool_diesel::postgres::{Object, Pool};

mod config;
mod error;
#[cfg(feature = "seeder")]
mod seeder;

pub mod controllers;
pub mod mailer;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod schema;

pub use config::Config;
pub use error::*;
use mailer::Mailer;
use redis::aio::MultiplexedConnection;
#[cfg(feature = "seeder")]
pub use seeder::*;

/// An entire database pool
pub type DbPool = Pool;

/// A single database connection
pub type DbConn = Object;

/// A redis cache connection
pub type RedisConn = MultiplexedConnection;

/// Common state of the app
#[derive(Clone)]
pub struct AppState {
	pub config:           Config,
	pub database_pool:    DbPool,
	pub redis_connection: RedisConn,
	pub cookie_jar_key:   Key,
	pub mailer:           Mailer,
}

impl FromRef<AppState> for Config {
	fn from_ref(input: &AppState) -> Self { input.config.clone() }
}

impl FromRef<AppState> for DbPool {
	fn from_ref(input: &AppState) -> Self { input.database_pool.clone() }
}

impl FromRef<AppState> for RedisConn {
	fn from_ref(input: &AppState) -> Self { input.redis_connection.clone() }
}

impl FromRef<AppState> for Key {
	fn from_ref(input: &AppState) -> Self { input.cookie_jar_key.clone() }
}

impl FromRef<AppState> for Mailer {
	fn from_ref(input: &AppState) -> Self { input.mailer.clone() }
}
