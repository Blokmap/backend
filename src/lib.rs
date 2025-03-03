#[macro_use]
extern crate tracing;

use deadpool_diesel::postgres::{Object, Pool};

pub mod config;
pub mod controllers;
pub mod error;
pub mod models;
pub mod schema;

pub type DbPool = Pool;
pub type DbConn = Object;
