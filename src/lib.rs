//! # Blokmap backend library

#[macro_use]
extern crate tracing;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use common::{DbPool, RedisConn};
use mailer::Mailer;

mod config;
mod seeder;
mod session;

pub mod controllers;
pub mod mailer;
pub mod middleware;
pub mod routes;
pub mod schemas;

pub use config::*;
pub use seeder::*;
pub use session::*;

/// Common state of the app
#[derive(Clone)]
pub struct AppState {
	pub config:           Config,
	pub sso_config:       SsoConfig,
	pub database_pool:    DbPool,
	pub redis_connection: RedisConn,
	pub cookie_jar_key:   Key,
	pub mailer:           Mailer,
}

impl FromRef<AppState> for Config {
	fn from_ref(input: &AppState) -> Self { input.config.clone() }
}

impl FromRef<AppState> for SsoConfig {
	fn from_ref(input: &AppState) -> Self { input.sso_config.clone() }
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
