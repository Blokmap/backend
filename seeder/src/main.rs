use clap::{Error, Parser};

#[derive(Parser, Debug)]
struct Opt {
	#[arg(long, default_value_t = 1_000, help = "Number of users to create")]
	profiles: usize,
}

#[tokio::main]
async fn main() -> Result<(), Error> { Ok(()) }
