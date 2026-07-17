// 中间件 与 Layer
use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use std::time::{Duration, Instant};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer, cors::{Any, CorsLayer}, trace::TraceLayer,
};
use tracing::Level;

// 日志中间件， 记录每个请求的日志
async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();
    let response = next.run(request).await;
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status().as_u16(),
        duration_ms = %start.elapsed().as_millis(),
        "Request completed"
    );
    response
}

// 计时中间件， 添加X-Response-Time 响应头
async fn timing_middleware(request: Request, next: Next) -> Response {
    // Instant 单调时间，不会回退，不受系统时间调整影响，一直递增
    let start = Instant::now();
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "X-Response-Time",
        HeaderValue::from_str(&format!("{}ms",start.elapsed().as_millis())).unwrap()
    );

    response
}

async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
    .headers()
    .get("X-API-KEY")
    .and_then(|v| v.to_str().ok());
    match auth_header {
        Some("secret-key") => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

// 内置 Tower-HTTP 中间件
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
}

// 处理器
async fn index() -> &'static str {
    "Welcome to Axum Middleware Module!"
}

async fn public_data() -> impl IntoResponse {
    axum::Json(serde_json::json!({"message":"Public data", "accessible":true}))
}

async fn protocted_data() -> impl IntoResponse {
    axum::Json(serde_json::json!({"nessage":"Secret data", "authorized":true}))
}

async fn slow_endpoint() -> &'static str {
    tokio::time::sleep(Duration::from_secs(1)).await;
    "Slow operation done!"
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // 初始化路由器
    // 受保护路由
    let protected = Router::new()
    .route("/data", get(protocted_data))
    .route_layer(middleware::from_fn(auth_middleware));

    // 带有分层中间件的主应用
    let app = Router::new()
    .route("/", get(index))
    .route("/public", get(public_data))
    .route("/slow", get(slow_endpoint))
    .nest("/protected", protected)
    .layer(middleware::from_fn(timing_middleware))
    .layer(middleware::from_fn(logging_middleware))
    .layer(ServiceBuilder::new()
              .layer(TraceLayer::new_for_http())
              .layer(cors_layer())
              .layer(CompressionLayer::new()),
    );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.expect("bind failed");
    println!("🚀 Module 06: Middleware & Layers");
    println!("   Server: http://localhost:3000");
    println!("\n📝 Endpoints:");
    println!("   GET /              - Welcome");
    println!("   GET /public        - Public data");
    println!("   GET /slow          - Slow endpoint");
    println!("   GET /protected/data - Auth required (X-API-Key: secret-key)");
    axum::serve(listener, app).await.expect("Server failed!");


}
