use axum::{ extract::State, http::StatusCode, routing::get, Json, Router };
use serde::{ Deserialize, Serialize };
use std::{ collections::HashMap, sync::{ Arc, RwLock } };

use uuid::Uuid;

// 运行时不会改变的配置
#[derive(Clone)]
struct AppConfig {
    app_name: String,
    version: String,
    max_items_per_page: usize,
}

async fn get_config(State(config): State<Arc<AppConfig>>) -> Json<serde_json::Value> {
    Json(
        serde_json::json!({
        "app_name": config.app_name,
        "version": config.version,
        "max_items": config.max_items_per_page,
    })
    )
}

// 使用 RwLock 以获得更好的读取性能（多读少写场景）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

// 可变状态：一个线程安全的HashMap
type TodoStore = Arc<RwLock<HashMap<String, Todo>>>;

// 列出所有 todo
async fn list_todos(State(store): State<TodoStore>) -> Json<Vec<Todo>> {
    let todos = store.read().unwrap();
    let todos_vec: Vec<Todo> = todos.values().cloned().collect();
    Json(todos_vec)
}

async fn create_todo(
    State(store): State<TodoStore>,
    Json(input): Json<CreateTodo>
) -> (StatusCode, Json<Todo>) {
    let todo = Todo {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        completed: false,
    };

    store.write().unwrap().insert(todo.id.clone(), todo.clone());
    (StatusCode::CREATED, Json(todo))
}

fn main() {
    println!("Hello, world!");
}
