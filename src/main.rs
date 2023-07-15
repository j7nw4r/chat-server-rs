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
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};
use serde::Deserialize;

use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast::{self, error::RecvError, Receiver, Sender};
use tracing::{debug, info};

#[derive(Debug, Clone)]
struct WsState {
    sender: Sender<ChatEvent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum ChatEvent {
    Message { user: String, content: String },
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let (sender, _) = broadcast::channel::<ChatEvent>(16);
    let state = Arc::new(WsState { sender });
    let router = Router::new()
        .route("/ws/chat", get(chat_ws_handler))
        .with_state(state)
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
    let (ws_writer, ws_reader) = socket.split();
    let (broadcast_sender, broadcast_receiver) = (state.sender.clone(), state.sender.subscribe());
    tokio::select! {
        _ = tokio::spawn(write_chat(ws_writer, broadcast_receiver)) => (),
        _ = tokio::spawn(read_chat(ws_reader, broadcast_sender)) => ()
    }
}

// For writing a message out.
async fn write_chat(
    mut writer: SplitSink<WebSocket, Message>,
    mut receiver: Receiver<ChatEvent>,
) -> anyhow::Result<()> {
    loop {
        let receive_result = receiver.recv().await;
        if let Err(RecvError::Lagged(_)) = receive_result {
            continue;
        };

        let message = match receive_result? {
            ChatEvent::Message { user, content } => Message::Text(format!("{}: {}", user, content)),
        };
        writer.send(message).await?;
    }
}

// For reading a message in.
async fn read_chat(
    mut reader: SplitStream<WebSocket>,
    sender: Sender<ChatEvent>,
) -> anyhow::Result<()> {
    loop {
        match reader.next().await {
            Some(message) => {
                let message = message.context("read a broken message")?;
                let message_text = message.into_text()?;
                info!("received the message {}", message_text);

                let chat_event = serde_json::from_str::<ChatEvent>(&message_text)
                    .context("could not deserialize chat event")?;
                sender
                    .send(chat_event)
                    .context("could not send chat event to websocket sender")?;
            }
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
