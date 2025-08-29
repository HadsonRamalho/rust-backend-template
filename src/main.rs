pub mod controllers;
pub mod models;
pub mod routes;
pub mod schema;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = crate::routes::init_routes().await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3099").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}
