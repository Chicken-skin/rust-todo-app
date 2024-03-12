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

    let app = create_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // 127.0.0.1:3000 (localhost:3000)
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", addr);

    axum::serve(listener, app.into_make_service())
        .await // 非同期タスクはawaitされるまで実行されない
        .unwrap();
}

/// # create_app
/// This function create app and define routing
///
/// ## Return
/// * app route: Router
fn create_app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
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
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct CreateUser {
    username: String,
}

/// # User
/// This struct is used for response.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct User {
    id: u64,
    username: String,
}

// test
#[cfg(test)]
mod test {
    use super::*;
    use axum::{
        body::{to_bytes, Body, Bytes},
        http::{header, Method, Request},
    };
    use hyper::body;
    use tower::ServiceExt;

    // root関数のtest
    #[tokio::test]
    async fn should_return_hello_world() {
        // request作成
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app().oneshot(req).await.unwrap();
        let bytes = axum::body::to_bytes(res.into_body(), 128).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(body, "Hello, world!");
    }

    // JSON bodyをtest
    #[tokio::test]
    async fn should_return_user_data() {
        // request作成
        let req = Request::builder()
            .uri("/users")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(r#"{ "username": "Phil Foden" }"#))
            .unwrap();
        let res = create_app().oneshot(req).await.unwrap();

        let bytes = axum::body::to_bytes(res.into_body(), 128).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let user: User = serde_json::from_str(&body).expect("cannot convert User instance.");
        // UserがPartialEqを実装しているので比較可能.
        assert_eq!(
            user,
            User {
                id: 1337,
                username: "Phil Foden".to_string(),
            }
        );
    }
}
