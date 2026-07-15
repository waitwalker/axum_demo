use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use uuid::Uuid;

// 运行时不会改变的配置
#[derive(Clone)]
struct AppConfig {
    app_name: String,
    version: String,
    max_items_per_page: usize,
}

async fn get_config(State(config): State<Arc<AppConfig>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "app_name": config.app_name,
        "version": config.version,
        "max_items": config.max_items_per_page,
    }))
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
    Json(input): Json<CreateTodo>,
) -> (StatusCode, Json<Todo>) {
    let todo = Todo {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        completed: false,
    };

    store.write().unwrap().insert(todo.id.clone(), todo.clone());
    (StatusCode::CREATED, Json(todo))
}

// 获取单个 todo
async fn get_todo(
    State(store): State<TodoStore>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Todo>, StatusCode> {
    let todos = store.read().unwrap();
    todos
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

// 更新TODO
async fn update_todo(
    State(store): State<TodoStore>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(input): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = store.write().unwrap();
    if let Some(todo) = todos.get_mut(&id) {
        if let Some(title) = input.title {
            todo.title = title;
        }
        if let Some(completed) = input.completed {
            todo.completed = completed;
        }
        Ok(Json(todo.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// 删除 todo
async fn delete_todo(
    State(store): State<TodoStore>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> StatusCode {
    let mut todos = store.write().unwrap();
    if todos.remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Clone)]
#[allow(dead_code)]
struct CombinedState {
    config: Arc<AppConfig>,
    todos: TodoStore,
    metrics: Arc<RwLock<Mertics>>,
}

#[derive(Debug, Default)]
struct Mertics {
    request_count: u64,
    error_count: u64,
}

// 提取整个状态，或者为了方便使用 From trait
async fn get_mertics(State(state): State<CombinedState>) -> Json<serde_json::Value> {
    let metrics = state.metrics.read().unwrap();
    Json(serde_json::json!({
        "requests": metrics.request_count,
        "errors": metrics.error_count,
        "app_version": state.config.version,
    }))
}

async fn increment_request_count(State(state): State<CombinedState>) -> &'static str {
    let mut metrics = state.metrics.write().unwrap();
    metrics.request_count += 1;
    "Request counted"
}

// 模拟数据库连接池
// 实际应用中，这应当是 sqlx::PgPool或类似的库
#[derive(Clone)]
#[allow(dead_code)]
struct DbPool {
    connection_string: String,
    max_connections: u32,
}

impl DbPool {
    fn new(connection_string: &str) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            max_connections: 10,
        }
    }

    // 模拟查询
    async fn query(&self, _sql: &str) -> Result<Vec<String>, String> {
        // 在真实数据库中 sqlx::query!(...).fetch_all(&self.pool).await
        Ok(vec!["result1".to_string(), "result2".to_string()])
    }
}

async fn db_query(State(pool): State<DbPool>) -> Json<Vec<String>> {
    match pool.query("SELECT * FROM users").await {
        Ok(results) => Json(results),
        Err(_) => Json(vec![]),
    }
}
use axum::Extension;
#[derive(Clone)]
struct CurrentUser {
    id: String,
    name: String,
}

async fn get_current_user(Extension(user): Extension<CurrentUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": user.id,
        "name": user.name,
    }))
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    // 初始化不可变配置
    let config = Arc::new(AppConfig {
        app_name: "Axum Todo API".to_string(),
        version: "1.0.0".to_string(),
        max_items_per_page: 100,
    });

    // 初始化可变 todo 存储
    let todo_store: TodoStore = Arc::new(RwLock::new(HashMap::new()));

    // 预先填充一些 todo
    {
        let mut store = todo_store.write().unwrap();
        let todo = Todo {
            id: Uuid::new_v4().to_string(),
            title: "Learn Axum".to_string(),
            completed: false,
        };
        store.insert(todo.id.clone(), todo);
    }

    // 用于复杂应用的组合状态
    let combined_state = CombinedState {
        config: config.clone(),
        todos: todo_store.clone(),
        metrics: Arc::new(RwLock::new(Mertics::default())),
    };

    // 模拟数据库连接池
    let db_pool = DbPool::new("postgres://localhost/myapp");

    // 当前用户（通常由认证中间件设置）
    let current_user = CurrentUser {
        id: "user-123".to_string(),
        name: "Demo User".to_string(),
    };

    // 构建 todo CRUD 路由
    let todo_routes = Router::new()
        .route("/", get(list_todos).post(create_todo))
        .route("/{id}", get(get_todo).put(update_todo).delete(delete_todo))
        .with_state(todo_store);

    // 构建主应用
    let app = Router::new()
        // 配置端点
        .route("/config", get(get_config))
        .with_state(config)
        // 合并路由
        .merge(Router::new().nest("/todos", todo_routes))
        //指标端点
        .route("/metrics", get(get_mertics))
        .route("/track", get(increment_request_count))
        .with_state(combined_state)
        // 数据库端点
        .route("/db/users", get(db_query))
        .with_state(db_pool)
        // 基于Extension的状态
        .route("/me", get(get_current_user))
        .layer(Extension(current_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    println!("🚀 Module 05: State Management");
    println!("   Server running on http://localhost:3000");
    println!();
    println!("📝 Todo CRUD Endpoints:");
    println!("   GET    /todos      - List all todos");
    println!("   POST   /todos      - Create todo");
    println!("   GET    /todos/:id  - Get single todo");
    println!("   PUT    /todos/:id  - Update todo");
    println!("   DELETE /todos/:id  - Delete todo");
    println!();
    println!("📝 Other Endpoints:");
    println!("   GET /config   - App configuration");
    println!("   GET /metrics  - Request metrics");
    println!("   GET /me       - Current user (Extension)");
    println!();
    println!("💡 Try: curl -X POST -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"title\":\"New Todo\"}}' http://localhost:3000/todos");

    axum::serve(listener, app).await.expect("Server failed");
}
