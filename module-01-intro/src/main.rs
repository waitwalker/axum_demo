// 这里的大括号是合并路径，和下面写法一样
use axum::{
    Router,
    routing::{get, post},
};

// use axum::routing::get;
// use axum::routing::post;
// use axum::Router;

async fn hello_world() -> &'static str {
    println!("Hello World 访问了");
    "Hello, World! 🦀"
}

async fn hello_axum() -> String {
    println!("Hello Axum 访问了");
    format!("Welcome to Axum {}!", env!("CARGO_PKG_VERSION"))
}

async fn health_check() -> &'static str {
    println!("Health check 访问了");
    "OK"
}

use axum::http::StatusCode;

async fn with_status() -> (StatusCode, &'static str) {
    println!("With status 访问了");
    (StatusCode::CREATED, "Resource created!")
}

async fn conditional_response() -> (StatusCode, &'static str) {
    println!("Conditional response 访问了");
    let is_working = true;
    if is_working {
        (StatusCode::OK, "Everything is working!")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Service is down")
    }
}

async fn echo(body: String) -> String {
    format!("You sent: {}", body)
}
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/hello", get(hello_axum))
        .route("/health", get(health_check))
        .route("/created", get(with_status))
        .route("/status", get(conditional_response))
        .route("/echo", post(echo));

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => {
            println!("OK");
            listener
        }
        Err(error) => {
            eprintln!("bind failed:{}", error);
            return;
        }
    };

    println!("🚀 Module 01: Introduction to Axum");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 Try these endpoints:");
    println!("   GET  /        - Hello World");
    println!("   GET  /hello   - Welcome message");
    println!("   GET  /health  - Health check");
    println!("   GET  /created - Status code example");
    println!("   POST /echo    - Echo back your message");
    println!();
    println!("💡 Example: curl http://localhost:3000/hello");
    println!("💡 Example: curl -X POST -d 'Hello!' http://localhost:3000/echo");

    match axum::serve(listener, app).await {
        Ok(_) => {
            println!("OK");
        }
        Err(error) => {
            eprintln!("fail{}", error);
        }
    };
}
