use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;

use crate::state::AppState;

/// Fan-out of chat messages to all connected sockets.
#[derive(Clone)]
pub struct ChatHub {
    tx: broadcast::Sender<String>,
}

impl ChatHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx }
    }

    fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    fn broadcast(&self, message: String) {
        // Fails only when nobody is listening.
        let _ = self.tx.send(message);
    }
}

impl Default for ChatHub {
    fn default() -> Self {
        Self::new()
    }
}

// route function to nest endpoints in router
pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

/// Documentation-only description of the chat WebSocket endpoint (OpenAPI cannot model WebSocket messages).
///
/// After the upgrade, the client must send its requester token as the first text message; the
/// server closes the connection if it is invalid. Every later text message is broadcast to all
/// connected clients as `"<username>: <message>"`.
#[utoipa::path(
    get,
    path = "/ws",
    tag = "chat",
    responses((status = 101, description = "Switching protocols to WebSocket"))
)]
#[allow(dead_code)]
pub fn ws_docs() {}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // The first text message authenticates the connection.
    let username = loop {
        match receiver.next().await {
            Some(Ok(Message::Text(token))) => {
                match state.auth.user_for_requester_token(&token).await {
                    Ok(user) => break user.username,
                    Err(_) => {
                        let _ = sender.close().await;
                        return;
                    }
                }
            }
            Some(Ok(_)) => continue,
            _ => return,
        }
    };

    let mut rx = state.chat.subscribe();
    state.chat.broadcast(format!("{username} joined."));

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if msg.is_empty() {
                break;
            }
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let chat = state.chat.clone();
    let name = username.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if text.is_empty() {
                break;
            }
            chat.broadcast(format!("{name}: {text}"));
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    state.chat.broadcast(format!("{username} left."));
}
