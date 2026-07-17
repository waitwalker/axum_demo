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
    classify::GrpcCode::Ok, compression::CompressionLayer, cors::{Any, CorsLayer}, trace::TraceLayer,
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

fn main() {
    println!("Hello, world!");
}
