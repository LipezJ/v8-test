mod utils;
mod routes;
mod ejecutor;

use axum::{routing, Router};

const TIMEOUT: u64 = 100;
const POOL_SIZE: u32 = 100;
const HEAP_LIMITS: (usize, usize) = (1000000, 2000000);

#[tokio::main]
async fn main() {
	ejecutor::init_ejecutor();
	tracing_subscriber::fmt::init();

	let app = Router::new()
		.route("/runner", routing::get(routes::runner));

	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}
