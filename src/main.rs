use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // loggingの初期化
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root)) // ルーティング
        .route("/users", post(create_user));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // 127.0.0.1:3000 (localhost:3000)
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app.into_make_service())
        .await // 非同期タスクはawaitされるまで実行されない
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}

/// # create_user
/// This function create user and return response.
///
/// ## Arguments
/// * Json data of CreateUser struct.
///
/// ## Return
/// * Something it is impl IntoResponse trait
async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // ここでSerialize
    // http status codeはCREATED(201)
    // response BodyはuserをJSON Serializeしたものをレスポンスに含める
    (StatusCode::CREATED, Json(user)) // StatusCodeのみを返すこともできる
}

/// # CreateUser
/// This struct is used for request.
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

/// # User
/// This struct is used for response.
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
