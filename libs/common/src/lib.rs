#[macro_use]
extern crate tracing;

use deadpool_diesel::postgres::{Object, Pool};
use redis::aio::MultiplexedConnection;

mod error;

pub use error::*;

/// An entire database pool
pub type DbPool = Pool;

/// A single database connection
pub type DbConn = Object;

/// A redis cache connection
pub type RedisConn = MultiplexedConnection;
