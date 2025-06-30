
use tokio::net::TcpListener;
use axum::{Router};
use routes::router;

mod routes;
mod handlers;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().merge(router()); // Use your actual router fn

    let addr: std::net::SocketAddr = "0.0.0.0:8080".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("🚀 Server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

