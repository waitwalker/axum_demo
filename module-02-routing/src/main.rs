use axum::{
    Router,
    extract::{Path, Query},
    routing::{delete, get, patch, post, put},
};

use serde::Deserialize;

async fn get_user(Path(id): Path<u64>) -> String {
    format!("Getting user with ID: {}", id)
}

async fn get_user_post(Path((user_id, post_id)): Path<(u64, u64)>) -> String {
    format!("User {} - Post {}", user_id, post_id)
}

#[derive(Deserialize)]
struct PostPath {
    user_id: u64,
    post_id: u64,
    comment_id: u64,
}

async fn get_comment(Path(params): Path<PostPath>) -> String {
    format!(
        "User {} - Post {} - Comment {}",
        params.user_id, params.post_id, params.comment_id
    )
}

async fn files(Path(path): Path<String>) -> String {
    format!("Accessing file:{}", path)
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_items(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(10);
    format!("Listing items - Page:{}, Limit: {}", page, limit)
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    category: Option<String>,
    sort: Option<String>,
}

async fn search(Query(params): Query<SearchParams>) -> String {
    format!(
        "Searching for '{}' in categor '{}', sorted by '{}'",
        params.q,
        params.category.unwrap_or_else(|| "all".to_string()),
        params.sort.unwrap_or_else(|| "relevance".to_string())
    )
}

async fn create_user() -> &'static str {
    "Creating new user (POST)"
}

async fn update_user(Path(id): Path<u64>) -> String {
    format!("Updating user {} (Put -full update)", id)
}

async fn patch_user(Path(id): Path<u64>) -> String {
    format!("Patching user {} (PATCH - partial update)", id)
}

async fn delete_user(Path(id): Path<u64>) -> String {
    format!("Deleting user {} (DELETE", id)
}

// 嵌套路由
// 创建一个用于用相关路由的子路由
fn user_routes() -> Router {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route(
            "/{id}",
            get(list_users)
                .put(update_user)
                .patch(patch_user)
                .delete(delete_user),
        )
}

async fn list_users() -> &'static str {
    "Listing all users"
}

// 创建一个用于帖子相关路由的子路由
fn post_routes() -> Router {
    Router::new()
        .route("/", get(list_posts))
        .route("/{id}", get(get_post))
}

async fn list_posts() -> &'static str {
    "Listing all posts"
}

async fn get_post(Path(id): Path<u64>) -> String {
    format!("Getting post {}", id)
}

// 合并路由
// 通过合并多个路由器来创建API v1 路由器
fn api_v1_routes() -> Router {
    Router::new()
        .nest("/users", user_routes())
        .nest("/posts", post_routes())
}

fn api_v2_routes() -> Router {
    Router::new()
        .route("/users", get(|| async { "API v2 Users endpoint" }))
        .route("/posts", get(|| async { "API v2 - Posts endpoint" }))
}

async fn not_found() -> (axum::http::StatusCode, &'static str) {
    (axum::http::StatusCode::NOT_FOUND,"404 - Route not found")
}
#[tokio::main]
async fn main() {
    let router = Router::new()
    .route("/", get(|| async {"Welcome to the Routing Module!"}))
    .route("/resource", get(|| async {"GET - Read resource"}))
    .route("/resource", post(|| async {"POST - Create resource"}))
    .route("/resource/{id}", get(|Path(id):Path<u64>| async move {format!("GET - Read resource {}", id)}))
    .route("/resource/{id}", put(|Path(id):Path<u64>| async move {format!("PUT - Full update resource {}", id)}))
    .route("/resource/{id}", patch(|Path(id):Path<u64>| async move {format!("PATCH - Partial update resource {}", id)}))
    .route("/resource/{id}", delete(|Path(id):Path<u64>| async move {format!("DELETE - Remove resource {}", id)}))

    // 路径参数
    .route("/users/{id}/posts/{post_id}", get(get_user_post))
    .route("/users/{user_id}/posts/{post_id}/comments/{comment_id}", get(get_comment))

    // 通配符
    .route("/files/{*path}", get(files))

    // Query 参数
    .route("/items", get(list_items))
    .route("/search", get(search))

    // 嵌套路由器
    .nest("/api/v1", api_v1_routes())
    .nest("/api/v2", api_v2_routes())

    // 未找到路由，回退处理
    .fallback(not_found);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
    .await
    .expect("Failed to bind to port 3000");

    println!("🚀 Module 02: Routing Deep Dive");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 HTTP Methods Demo:");
    println!("   GET    /resource      - Read all");
    println!("   POST   /resource      - Create new");
    println!("   GET    /resource/123  - Read one");
    println!("   PUT    /resource/123  - Full update");
    println!("   PATCH  /resource/123  - Partial update");
    println!("   DELETE /resource/123  - Remove");
    println!();
    println!("📝 Path Parameters (NEW SYNTAX!):");
    println!("   GET /users/123/posts/456");
    println!("   GET /files/docs/readme.md");
    println!();
    println!("📝 Nested Routes:");
    println!("   GET  /api/v1/users");
    println!("   POST /api/v1/users");
    println!("   PUT  /api/v1/users/123");

    axum::serve(listener, router)
    .await
    .expect("Server failed");

}
