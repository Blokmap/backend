use std::sync::{LazyLock, Mutex, MutexGuard};

use redis::aio::MultiplexedConnection;
use redis::cmd;

const REDIS_CONNECTIONS_LEN: usize = 16;

/// List of available redis URLs
pub static REDIS_CONNECTION_URLS: LazyLock<
	[Mutex<&'static str>; REDIS_CONNECTIONS_LEN],
> = LazyLock::new(|| {
	[
		Mutex::new("redis://127.0.0.1:6379/0"),
		Mutex::new("redis://127.0.0.1:6379/1"),
		Mutex::new("redis://127.0.0.1:6379/2"),
		Mutex::new("redis://127.0.0.1:6379/3"),
		Mutex::new("redis://127.0.0.1:6379/4"),
		Mutex::new("redis://127.0.0.1:6379/5"),
		Mutex::new("redis://127.0.0.1:6379/6"),
		Mutex::new("redis://127.0.0.1:6379/7"),
		Mutex::new("redis://127.0.0.1:6379/8"),
		Mutex::new("redis://127.0.0.1:6379/9"),
		Mutex::new("redis://127.0.0.1:6379/10"),
		Mutex::new("redis://127.0.0.1:6379/11"),
		Mutex::new("redis://127.0.0.1:6379/12"),
		Mutex::new("redis://127.0.0.1:6379/13"),
		Mutex::new("redis://127.0.0.1:6379/14"),
		Mutex::new("redis://127.0.0.1:6379/15"),
	]
});

/// A RAII guard provider to lock down a single redis URL
pub struct RedisUrlProvider;

/// A RAII guard for a single redis URL
pub struct RedisUrlGuard(MutexGuard<'static, &'static str>);

impl RedisUrlProvider {
	/// Get a locked redis URL
	pub fn acquire() -> RedisUrlGuard {
		RedisUrlGuard(Self::get_redis_connection_url())
	}

	/// Loop over all available URLs until a free one is found
	fn get_redis_connection_url() -> MutexGuard<'static, &'static str> {
		let mut i = 0;
		loop {
			let mutex = &REDIS_CONNECTION_URLS[i];

			if let Ok(lock) = mutex.try_lock() {
				return lock;
			}

			i = (i + 1) % REDIS_CONNECTIONS_LEN;
		}
	}
}

impl RedisUrlGuard {
	/// Connect to this locked URL
	pub async fn connect(&self) -> MultiplexedConnection {
		let client = redis::Client::open(*self.0).unwrap();
		client.get_multiplexed_async_connection().await.unwrap()
	}
}

impl Drop for RedisUrlGuard {
	fn drop(&mut self) {
		futures::executor::block_on(async {
			let mut conn = self.connect().await;

			let _: bool = cmd("FLUSHDB").query_async(&mut conn).await.unwrap();
		});
	}
}
