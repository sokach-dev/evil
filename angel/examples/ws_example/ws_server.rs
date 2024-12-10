use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use tokio::{net::TcpListener, sync::broadcast};

#[tokio::main]
async fn main() {
    // 创建一个广播通道
    let (tx, _rx) = broadcast::channel(100);

    // 设置路由，绑定 WebSocket 处理函数
    let app = Router::new().route(
        "/ws",
        get(move |ws: WebSocketUpgrade| handle_ws(ws, tx.clone())),
    );

    let addr = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(addr, app).await.unwrap();
}

// WebSocket 升级处理函数
async fn handle_ws(ws: WebSocketUpgrade, tx: broadcast::Sender<String>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

// WebSocket 消息处理函数
async fn handle_socket(socket: WebSocket, tx: broadcast::Sender<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    // 接收客户端消息
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // 广播消息给其他客户端
                tx.send(text).unwrap();
            }
        }
    });

    // 发送消息给客户端
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
}
