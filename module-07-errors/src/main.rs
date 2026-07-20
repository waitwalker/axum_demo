use axum::{
    extract::path,
    http::StatusCode,
    response::{ IntoResponse, Response },
    routing::get,
    Json,
    Router,
};

use serde::Serialize;
use thiserror::Error;

// 通过thiserror自定义错误类型

#[derive(Error, Debug)]
#[allow(dead_code)]
enum AppError {
    #[error("User not found: {0}")] UserNotFound(u64),

    #[error("Invalid input: {0}")] InvalidInput(String),

    #[error("Database error: {0}")] DatabaseError(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Internal server error")]
    Internal,
}

// 为自定义错误实现 IntoResponse
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: u16,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::UserNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidInput(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = ErrorResponse {
            error: message,
            code: status.as_u16(),
        };
        (status, Json(body)).into_response()
    }
}

// 基于 Result 的处理器
#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
}

async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, AppError> {
    // 模拟查找用户
    match id {
        1 => Ok(Json(User { id: 1, name: "Alice".to_string() })),
        2 => Ok(Json(User { id: 2, name: "Bod".to_string() })),
        _ => Err(AppError::UserNotFound(id))
    }
}

fn main() {
    println!("Hello, world!");
}
