use axum::{
    extract::Path,
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

async fn avlidate_input(Path(value): Path<String>) -> Result<String, AppError> {
    if value.len() < 3 {
        return Err(AppError::InvalidInput("Value must be at least 3 characters".to_string()));
    }
    Ok(format!("Valid input: {}", value))
}

async fn protected_resource() -> Result<&'static str, AppError> {
    // 模拟认证
    let is_authenticated = false;
    if !is_authenticated {
        return Err(AppError::Unauthorized);
    }
    Ok("Secret data!")
}

async fn database_operation() -> Result<&'static str, AppError> {
    // 模拟数据库报错
    Err(AppError::DatabaseError("Connection timeout".to_string()))
}

// 使用?处理简易的错误
async fn complex_operation(Path(id):Path<u64>) -> Result<Json<User>, AppError> {
    let user = find_user(id)?;
    validate_user(&user)?;
    Ok(Json(user))
}

fn find_user(id: u64) -> Result<User, AppError> {
    if id == 0 {
        Err(AppError::InvalidInput("ID cannot be zero".to_string()))
    } else if id > 100 {
        Err(AppError::UserNotFound(id))
    } else {
        Ok(User {
            id,
            name: format!("User{}", id)
        })
    }
}

fn validate_user(user: &User) -> Result<(), AppError> {
    if user.name.is_empty() {
        Err(AppError::InvalidInput("Name cannot be empty".to_string()))
    } else {
        Ok(())
    }
}

fn main() {
    println!("Hello, world!");
}
