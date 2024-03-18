use axum::{
    async_trait,
    extract::{Extension, FromRequest, Path, Request},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use validator::Validate;

use crate::repositories::{CreateTodo, TodoRepository, UpdateTodo};

#[derive(Debug)]
pub struct ValidatedJson<T>(T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &B) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                let message = format!("Json parse error: [{}]", rejection);
                (StatusCode::BAD_REQUEST, message)
            })?;
        value.validate().map_err(|rejection| {
            let message = format!("Validation error: [{}]", rejection).replace('\n', ", ");
            (StatusCode::BAD_REQUEST, message)
        })?;
        Ok(ValidatedJson(value))
    }
}

// todoを作成
pub async fn create_todo<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<CreateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository
        .create(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::CREATED, Json(todo)))
}

// 指定したidのtodoを取得
pub async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>, // pathにi32を含む場合はこのように書くとidを受け取れる
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.find(id).await.or(Err(StatusCode::NOT_FOUND))?; // find失敗でNotFound
    Ok((StatusCode::OK, Json(todo)))
}

// todoを全て取得しvector型で返す.
pub async fn all_todo<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.all().await.unwrap();
    Ok((StatusCode::OK, Json(todo)))
}

// todoをupdate
pub async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
    ValidatedJson(payload): ValidatedJson<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository
        .update(id, payload)
        .await
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
        .await
        .map(|_| StatusCode::NO_CONTENT) // 戻り値のハンドリング
        .unwrap_or(StatusCode::NOT_FOUND) // 戻り値のハンドリング
}
