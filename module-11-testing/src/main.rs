// 单元测试
// 测试 Axum 应用程序
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

type UserStore = Arc<RwLock<HashMap<u64, User>>>;

async fn list_users(State(store): State<UserStore>) -> Json<Vec<User>> {
    let users = store.read().unwrap();
    Json(users.values().cloned().collect())
}

async fn get_user(
    State(store): State<UserStore>,
    Path(id): Path<u64>,
) -> Result<Json<User>, StatusCode> {
    let users = store.read().unwrap();
    users
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_user(
    State(store): State<UserStore>,
    Json(input): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    let mut users = store.write().unwrap();
    let id = (users.len() as u64) + 1;
    let user = User {
        id,
        name: input.name,
    };
    users.insert(id, user.clone());
    (StatusCode::CREATED, Json(user))
}

async fn health() -> &'static str {
    "OK"
}

fn create_app(store: UserStore) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", get(get_user))
        .with_state(store)
}

// 测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_store() -> UserStore {
        Arc::new(RwLock::new(HashMap::new()))
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = create_app(test_store());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"OK");
    }
}

fn main() {
    println!("Hello, world!");
}
