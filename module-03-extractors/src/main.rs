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

async fn create_user(Json(payload): Json<CreateUserRequest>) -> Json<CreateUserResponse> {
    Json(CreateUserResponse {
        id: 1,
        name: payload.name,
        email: payload.email,
    })
}

async fn show_headers(headers: HeaderMap) -> String {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unkonwn");

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Not specified");
    format!(
        "User-Agent:{} \n Content-Type: {}",
        user_agent, content_type
    )
}

// 请求体（Raw body）提取器
async fn raw_body(body: Bytes) -> String {
    format!("Received {} bytes", body.len())
}

async fn combined_extractors(
    Path(id): Path<u64>,
    Query(params): Query<ListParams>,
    headers: HeaderMap,
    Json(body): Json<CreateUserRequest>,
) -> String {
    format!(
        "ID: {}\nPage: {:?}\nUser-Agent: {:?}\nName: {}",
        id,
        params.page,
        headers.get("user-agent"),
        body.name
    )
}

async fn optional_query(Query(params): Query<Option<ListParams>>) -> String {
    match params {
        Some(p) => format!("Got params: page={:?}", p.page),
        None => "No query params provided".to_string(),
    }
}

struct ApiKey(String);

#[derive(Debug)]
struct ApiKeyError;

impl IntoResponse for ApiKeyError {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            "Missiing or invalid API key. Provide X-API-Key header",
        )
            .into_response()
    }
}

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = ApiKeyError;
    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Out = Result<Self, self::Rejection>> + Send {
        let api_key = parts
            .headers
            .get("x-api-key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        async move {
            match api_key {
                Some(key) if !key.is_empty() => Ok(ApiKey(key)),
                _ => Err(ApiKeyError),
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
