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
        .unwrap_or("Unknown");

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
            "Missing or invalid API key. Provide X-API-Key header",
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
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
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

async fn protected_endpoint(ApiKey(key): ApiKey) -> String {
    format!("Access granted! Your API key: {}", key)
}

// 一个验证 JSON 请求体的自定义提取器
#[derive(Debug, Deserialize)]
struct ValidatedUser {
    name: String,
    email: String,
}

struct ValidatedJson<T>(T);

#[derive(Debug)]
enum ValidationError {
    InvalidJson(String),
    InvalidEmail,
    NameTooShort,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ValidationError::InvalidJson(value) => {
                (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", value))
            }
            ValidationError::InvalidEmail => {
                (StatusCode::BAD_REQUEST, "Invalid email format".to_string())
            }
            ValidationError::NameTooShort => (
                StatusCode::BAD_REQUEST,
                "Name must be at least 2 characters".to_string(),
            ),
        };
        (status, message).into_response()
    }
}

impl<S> FromRequest<S> for ValidatedJson<ValidatedUser>
where
    S: Send + Sync,
{
    type Rejection = ValidationError;

    fn from_request(
        req: Request,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let Json(user): Json<ValidatedUser> = Json::from_request(req, state)
                .await
                .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;

            if user.name.len() < 2 {
                return Err(ValidationError::NameTooShort);
            }

            if !user.email.contains("@") {
                return Err(ValidationError::InvalidEmail);
            }

            Ok(ValidatedJson(user))
        }
    }
}

async fn create_validated_user(ValidatedJson(user): ValidatedJson<ValidatedUser>) -> String {
    format!("Created user: {} <{}>", user.name, user.email)
}

#[derive(Clone)]
struct AppState {
    db_pool: String,
    api_version: String,
}

async fn with_state(State(state): State<Arc<AppState>>) -> String {
    format!("API Version: {}, DB: {}", state.api_version, state.db_pool)
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let state = Arc::new(AppState {
        db_pool: "postgres://localhost/mydb".to_string(),
        api_version: "v1.0.0".to_string(),
    });

    let router: Router = Router::new()
        .route("/users/{id}", get(get_user))
        .route("/users", get(list_users).post(create_user))
        .route("/headers", get(show_headers))
        .route("/raw", post(raw_body))
        .route("/users/{id}/update", post(combined_extractors))
        .route("/optional", get(optional_query))
        .route("/protected", get(protected_endpoint))
        .route("/validated", post(create_validated_user))
        .route("/state", get(with_state))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    println!("🚀 Module 03: Extractors Deep Dive");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 Built-in Extractors:");
    println!("   GET  /users/123          - Path extractor");
    println!("   GET  /users?page=2       - Query extractor");
    println!("   POST /users              - Json extractor");
    println!("   GET  /headers            - Headers extractor");
    println!();
    println!("📝 Custom Extractors:");
    println!("   GET  /protected          - API key (Header: X-API-Key)");
    println!("   POST /validated          - Validated JSON body");
    println!();
    println!("💡 Examples:");
    println!("   curl http://localhost:3000/users?page=2&limit=5");
    println!("   curl -H 'X-API-Key: secret123' http://localhost:3000/protected");
    println!("   curl -X POST -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"name\":\"John\",\"email\":\"john@example.com\"}}' \\");
    println!("        http://localhost:3000/validated");

    axum::serve(listener, router).await.expect("Server failed");
}
