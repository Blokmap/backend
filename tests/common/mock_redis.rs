use std::sync::{LazyLock, Mutex, MutexGuard};

use redis::aio::MultiplexedConnection;
use redis::cmd;

const REDIS_CONNECTIONS_LEN: usize = 16;

pub static REDIS_CONNECTION_URLS: LazyLock<
	[Mutex<&'static str>; REDIS_CONNECTIONS_LEN],
> = LazyLock::new(|| {
	let redis_url = std::env::var("REDIS_URL").unwrap();

	[
		Mutex::new(format!("{redis_url}/0").leak()),
		Mutex::new(format!("{redis_url}/1").leak()),
		Mutex::new(format!("{redis_url}/2").leak()),
		Mutex::new(format!("{redis_url}/3").leak()),
		Mutex::new(format!("{redis_url}/4").leak()),
		Mutex::new(format!("{redis_url}/5").leak()),
		Mutex::new(format!("{redis_url}/6").leak()),
		Mutex::new(format!("{redis_url}/7").leak()),
		Mutex::new(format!("{redis_url}/8").leak()),
		Mutex::new(format!("{redis_url}/9").leak()),
		Mutex::new(format!("{redis_url}/10").leak()),
		Mutex::new(format!("{redis_url}/11").leak()),
		Mutex::new(format!("{redis_url}/12").leak()),
		Mutex::new(format!("{redis_url}/13").leak()),
		Mutex::new(format!("{redis_url}/14").leak()),
		Mutex::new(format!("{redis_url}/15").leak()),
	]
});

pub struct RedisUrlLock;

pub struct RedisUrlGuard(MutexGuard<'static, &'static str>);

impl RedisUrlLock {
	/// Get a connection to a locked URL
	pub fn get() -> RedisUrlGuard {
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
