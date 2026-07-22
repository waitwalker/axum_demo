// 高级特性
// WebSockets, SSE, File uploads, Static files
use axum::{
    Router, extract::{
        Multipart, ws::{Message, WebSocket, WebSocketUpgrade},
    }, http::response, response::{
        Html, IntoResponse, sse::{Event, KeepAlive, Sse},
    }, routing::{get, post},
};
use futures::stream::{self, Stream};
use std::{convert::Infallible, time::Duration};
use tokio_stream::StreamExt;
use tower_http::services::ServeDir;

// WebSocket
async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await { 
        if let Ok(Message::Text(text)) = msg {
            let response = format!("Echo: {}", text);
            if socket.send(Message::Text(response.into())).await.is_err() {
                break;
            }
        }
    }
}

// 服务器发送事件SSE
async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(||{
        Event::default().data(format!("Server time: {:?}", std::time::SystemTime::now()))
    })
    .map(Ok)
    .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// 文件上传
async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let mut files = Vec::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("unknown").to_string();
        let data = field.bytes().await.unwrap();
        files.push(format!("{}: {} bytes", name, data.len()));
    }

    if files.is_empty() {
        "No files uploaded".to_string()
    } else {
        format!("Uploaded: {}", files.join(","))
    }
}


fn main() {
    println!("Hello, world!");
}
