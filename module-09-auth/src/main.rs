use axum::{
    extract::{ Request, State },
    http::StatusCode,
    middleware::{ self, Next },
    response::{ IntoResponse, Response },
    routing::{ get, post },
    Json,
    Router,
};

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// 配置
#[derive(Clone)]
struct AuthConfig {
    jwt_secret: String,
    jwt_expiry_hours: i64,
}

// 数据模型
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    role: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}
#[derive(Serialize)]
struct LoginResponse {
    token: String,
    expires_in: i64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone)]
struct CurrentUser {
    id: String,
    role: String,
}

// 密码哈希
fn hash_password(password: &str) -> String {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .unwrap()
    .to_string()
}

// 校验密码与哈希值
fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok()
}

// JWT(JSON Web Token)
fn create_token(config: &AuthConfig, user_id: &str, role: &str) -> Result<String, StatusCode> {
    let expiry = Utc::now() + Duration::hours(config.jwt_expiry_hours);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiry.timestamp() as usize,
        role: role.to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(config.jwt_secret.as_bytes()))
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn verify_token(config: &AuthConfig, token: &str) -> Result<Claims, StatusCode> {
    decode::<Claims>(token, &DecodingKey::from_secret(config.jwt_secret.as_bytes()), &Validation::default())
    .map(|data| data.claims)
    .map_err(|_| StatusCode::UNAUTHORIZED)
}

// 处理器
async fn register(Json(input): Json<RegisterRequest>) -> impl IntoResponse {
    let _hashed = hash_password(&input.password);
    Json(serde_json::json!({"message":"User registered", "email":&input.email}))
}

async fn login(State(config):State<Arc<AuthConfig>>, Json(input):Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
    // 模拟用户有效性校验
    if input.email == "test@example.com" && input.password == "password123" {
        let token = create_token(&config, "user-1", "user")?;
        Ok(Json(LoginResponse { token, expires_in: config.jwt_expiry_hours * 3600 }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn protected(axum::Extension(user):axum::Extension<CurrentUser>) -> impl IntoResponse {
    Json(serde_json::json!({"message":"Access granted", "user_id":user.id, "role":user.role}))
}

async fn admin_only(axum::Extension(user):axum::Extension<CurrentUser>) -> impl IntoResponse {
    if user.role != "admin" {
        return  (StatusCode::FORBIDDEN, "Admin access required").into_response();
    } 
    Json(serde_json::json!({"message":"Admin area", "user":user.id})).into_response()
}

// 中间件
async fn auth_middleware(State(config):State<Arc<AuthConfig>>, mut request:Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request.headers()
    .get("Authorization")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "));

    let token = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = verify_token(&config, token)?;
    let user = CurrentUser {
        id: claims.sub,
        role: claims.role,
    };
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}


#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let config = Arc::new(AuthConfig {
        jwt_secret: "super-secret-key-change-in-production".to_string(),
        jwt_expiry_hours : 24,
    });

    let protected_routes = Router::new()
    .route("/me", get(protected))
    .route("/admin", get(admin_only))
    .route_layer(middleware::from_fn_with_state(config.clone(), auth_middleware));

    let app = Router::new()
    .route("/register", post(register))
    .route("/login", post(login))
    .nest("/protected", protected_routes)
    .with_state(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.expect("bind failed");

    println!("🚀 Module 09: Authentication");
    println!("   Server: http://localhost:3000\n");
    println!("📝 Endpoints:");
    println!("   POST /register    - Register user");
    println!("   POST /login       - Login (test@example.com / password123)");
    println!("   GET  /protected/me - Protected route");
    println!("\n💡 Usage:");
    println!("   1. POST /login with credentials");
    println!("   2. Use token: curl -H 'Authorization: Bearer <token>' /protected/me");

    axum::serve(listener, app).await.expect("Server failed");
}
