mod utils;
mod routes;
mod runner;

use axum::{routing, Router};

const TIMEOUT: u64 = 100;
const POOL_SIZE: u32 = 100;
const HEAP_LIMITS: (usize, usize) = (1000000, 2000000);

#[tokio::main]
async fn main() {
	runner::init_ejecutor();
	tracing_subscriber::fmt::init();

	let app = Router::new()
		.route("/runner", routing::get(routes::runner));

	let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}
