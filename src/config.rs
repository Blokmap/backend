use deadpool_diesel::postgres::{Manager, Pool};

pub struct Config {
	pub database_url: String,
	pub server_host: String,
	pub server_port: u16,
}

impl Config {
	pub fn from_env() -> Self {
		let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
		let server_host = std::env::var("SERVER_HOST").expect("SERVER_HOST must be set.");
		let server_port = std::env::var("SERVER_PORT")
			.expect("SERVER_PORT must be set.")
			.parse::<u16>()
			.expect("SERVER_PORT must be a number.");

		Self { database_url, server_host, server_port }
	}

	pub async fn setup_database(&self) -> Pool {
		let manager = Manager::new(&self.database_url.clone(), deadpool_diesel::Runtime::Tokio1);
		let pool = Pool::builder(manager).build().unwrap();
		pool
	}
}
