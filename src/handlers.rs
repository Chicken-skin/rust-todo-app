use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::repositories::{self, CreateTodo, TodoRepository, UpdateTodo};

// todoを作成
pub async fn create_todo<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>,
    Json(payload): Json<CreateTodo>,
) -> impl IntoResponse {
    let todo = repository.create(payload);

    (StatusCode::CREATED, Json(todo))
}

// 指定したidのtodoを取得
pub async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>, // pathにi32を含む場合はこのように書くとidを受け取れる
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.find(id).ok_or(StatusCode::NOT_FOUND)?; // find失敗でNotFound
    Ok((StatusCode::OK, Json(todo)))
}

// todoを全て取得しvector型で返す.
pub async fn all_todo<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let todo = repository.all();
    (StatusCode::OK, Json(todo))
}

// todoをupdate
pub async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
    Json(payload): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository
        .update(id, payload)
        .or(Err(StatusCode::NOT_FOUND))?; // update失敗でNotFound
    Ok((StatusCode::CREATED, Json(todo)))
}

// todoを削除
pub async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    repository
        .delete(id) // return -> Result<()>
        .map(|_| StatusCode::NO_CONTENT) // 戻り値のハンドリング
        .unwrap_or(StatusCode::NOT_FOUND) // 戻り値のハンドリング
}
