use anyhow::Context;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router, Server,
};
use futures::{
    future::join,
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};
use std::{
    net::SocketAddr,
    process::{ExitCode, ExitStatus},
    sync::Arc,
};
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

#[derive(Debug, Clone)]
struct WsState {}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/ws/chat", get(chat_ws_handler))
        .with_state(Arc::new(WsState {}))
        .fallback(fallback_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], 23234));
    Server::bind(&addr)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn chat_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<WsState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_chat(socket, state))
}

async fn handle_chat(socket: WebSocket, state: Arc<WsState>) {
    let (writer, reader) = socket.split();
    let write_handler = tokio::spawn(write_chat(writer));
    let read_handler = tokio::spawn(read_chat(reader));
    tokio::select! {
        _ = write_handler => (),
        _ = read_handler => ()
    }
}

// For writing a message out.
async fn write_chat(mut writer: SplitSink<WebSocket, Message>) -> anyhow::Result<()> {
    loop {
        writer.send(Message::Text("testing".to_owned())).await?;
        sleep(Duration::from_secs(5)).await;
    }
}

// For reading a message in.
async fn read_chat(mut reader: SplitStream<WebSocket>) -> anyhow::Result<()> {
    fn process(message: Message) -> anyhow::Result<()> {
        let message_text = message.into_text()?;
        info!("received the message {}", message_text);
        Ok(())
    }

    loop {
        match reader.next().await {
            Some(message) => process(message.context("read a broken message")?)?,
            None => {
                debug!("websocket stream done");
                return Ok(());
            }
        }
    }
}

async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "endpoint not found. Try again")
}
