use axum::{
    Router,
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::get,
};
use serde::Serialize;

// response 响应 字符串切片
async fn static_string() -> &'static str {
    "Hello from static string!"
}

async fn owned_string() -> String {
    format!("Hellow at timestamp: {}", chrono_lite())
}

fn chrono_lite() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// 返回一个包含状态码的元祖
async fn with_status() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "Resource created successfully!")
}
#[derive(Serialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    active: bool,
}

async fn json_user() -> Json<User> {
    Json(User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        active: true,
    })
}
#[derive(Serialize)]
struct UsersResponse {
    users: Vec<User>,
    total: usize,
    page: u64,
}

async fn json_users() -> Json<UsersResponse> {
    let users = vec![
        User {
            id: 1,
            name: "Jhon".to_string(),
            email: "John@example.com".to_string(),
            active: true,
        },
        User {
            id: 2,
            name: "Jane".to_string(),
            email: "Jane@example.com".to_string(),
            active: true,
        },
    ];
    let total = users.len();
    Json(UsersResponse {
        users,
        total,
        page: 1,
    })
}

// 带自定义状态码的 JSON
async fn json_with_status() -> (StatusCode, Json<User>) {
    (StatusCode::CREATED,Json(User{
        id: 3,
        name: "New User".to_string(),
        email:"new@example.com".to_string(),
        active:true,
    }))
}

async fn html_page() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
        <html>
        <head>
            <title>Axum HTML Response</title>
            <style>
                body {
                    font-family: system-ui, sans-serif;
                    max-width: 800px;
                    margin: 50px auto;
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                }
                .card {
                    background: white;
                    border-radius: 12px;
                    padding: 30px;
                    box-shadow: 0 10px 40px rgba(0,0,0,0.2);
                }
                h1 { color: #333; }
                p { color: #666; line-height: 1.6; }
            </style>
        </head>
        <body>
            <div class="card">
                <h1>🦀 Welcome to Axum!</h1>
                <p>This is an HTML response from your Axum server.</p>
                <p>You can return full HTML pages, templates, or fragments.</p>
            </div>
        </body>
        </html>
    "#)
}

fn main() {
    println!("Hello, world!");
}
