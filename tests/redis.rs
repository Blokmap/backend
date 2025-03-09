mod common;

use common::get_test_env;
use redis::AsyncCommands;

#[tokio::test(flavor = "multi_thread")]
async fn test_a() {
	let env = get_test_env(true).await;

	let mut conn = env.redis_guard.connect().await;

	conn.set::<_, _, bool>("test", "1").await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_b() {
	let env = get_test_env(true).await;

	let mut conn = env.redis_guard.connect().await;

	let res = conn.get::<_, Option<String>>("test").await.unwrap();
	assert_eq!(res, None);
}
