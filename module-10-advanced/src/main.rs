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
use std::{convert::Infallible, format, time::Duration};
use tokio_stream::StreamExt;
use tower_http::{classify::GrpcCode::Ok, services::ServeDir};

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



fn main() {
    println!("Hello, world!");
}
