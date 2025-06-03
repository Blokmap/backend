//! # Blokmap backend library

#[macro_use]
extern crate tracing;

use std::ops::Deref;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use common::{DbPool, RedisConn};
use mailer::Mailer;

mod config;
mod seeder;

pub mod controllers;
pub mod mailer;
pub mod middleware;
pub mod routes;
pub mod schemas;

pub use config::*;
pub use seeder::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProfileId(pub(crate) i32);

impl Deref for ProfileId {
	type Target = i32;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl AsRef<i32> for ProfileId {
	fn as_ref(&self) -> &i32 { &self.0 }
}

impl std::fmt::Display for ProfileId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

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
