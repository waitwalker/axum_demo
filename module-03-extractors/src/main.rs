use axum::{
    Json, Router,
    body::Bytes,
    extract::{FromRequest, FromRequestParts, Path, Query, Request, State},
    http::{StatusCode, header::HeaderMap, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;

// 路径提取器， 提取路径参数
// GET /user/{id}
async fn get_user(Path(id): Path<u64>) -> String {
    format!("User ID: {}", id)
}

#[derive(Debug, Deserialize)]
struct ListParams {
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
}

async fn list_users(Query(params): Query<ListParams>) -> String {
    format!(
        "Page: {}, Limit: {}, Sort: {}",
        params.page.unwrap_or(1),
        params.limit.unwrap_or(10),
        params.sort.unwrap_or_else(|| "id".to_string())
    )
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Serialize)]
struct CreateUserResponse {
    id: u64,
    name: String,
    email: String,
}

async fn create_user(Json(payload):Json<CreateUserRequest>) -> Json<CreateUserResponse> {
    Json(CreateUserResponse { id: 1, name: payload.name, email: payload.email })
}



fn main() {
    println!("Hello, world!");
}
