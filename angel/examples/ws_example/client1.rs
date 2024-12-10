use futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    let url = url::Url::parse("ws://127.0.0.1:3000/ws").unwrap();
    let (mut socket, _) = connect_async(url.as_str()).await.expect("连接失败");

    // 发送消息到服务器
    socket
        .send(tokio_tungstenite::tungstenite::Message::Text(
            "来自客户端 1 的消息".into(),
        ))
        .await
        .unwrap();

    // 接收来自服务器的消息
    while let Some(msg) = socket.next().await {
        println!("客户端 1 收到: {}", msg.unwrap());
    }
}
