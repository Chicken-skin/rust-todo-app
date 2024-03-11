use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root)); // ルーティング
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // 127.0.0.1:3000 (localhost:3000)
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await // 非同期タスクはawaitされるまで実行されない
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}
