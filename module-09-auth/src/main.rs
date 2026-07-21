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


fn main() {
    println!("Hello, world!");
}
