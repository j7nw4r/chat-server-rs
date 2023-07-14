use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use futures::stream::{SplitSink, SplitStream, StreamExt};
use std::net::SocketAddr;
use tracing::debug;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/ws/chat", get(chat_ws_handler))
        .fallback(fallback_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], 23234));
    Server::bind(&addr)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn chat_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_chat)
}

async fn handle_chat(socket: WebSocket) {
    let (writer, reader) = socket.split();
    tokio::spawn(write_chat(writer));
    tokio::spawn(read_chat(reader));
}

async fn write_chat(mut writer: SplitSink<WebSocket, Message>) {}

async fn read_chat(mut reader: SplitStream<WebSocket>) -> anyhow::Result<()> {
    fn process(message: Message) {}

    loop {
        match reader.next().await {
            Some(message) => {
                match message {
                    Ok(message) => process(message),
                    Err(e) => {
                        debug!("message read from websocket reader was an error: {}", e);
                        continue;
                    }
                };
            }
            None => return Ok(()),
        }
    }
}

async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "endpoint not found. Try again")
}
