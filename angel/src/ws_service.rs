use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{stream::StreamExt, SinkExt};
use tokio::sync::broadcast;
use tracing::{error, info};

pub async fn ws_handler(ws: WebSocketUpgrade, tx: broadcast::Sender<String>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(mut socket: WebSocket, tx: broadcast::Sender<String>) {
    // 构建欢迎消息
    let welcome_message = {
        serde_json::json!({
            "type": "system",
            "event": "welcome",
            "data": {
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        })
        .to_string()
    };
    // 发送欢迎消息
    if let Err(e) = socket.send(Message::Text(welcome_message)).await {
        error!("Failed to send welcome message to: {e}");
        return;
    }

    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    // send_task
    let mut send_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if tx.send(text).is_err() {
                    break;
                }
            }
        }
    });

    // recv_task
    let mut recv_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    info!("Websocket context destroyed");
}
