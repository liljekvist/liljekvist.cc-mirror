use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::commands;

/// JSON payload the client sends over the WebSocket.
#[derive(Debug, Deserialize)]
struct WsRequest {
    /// The raw command line typed by the user.
    line: String,
}

/// JSON payload the server sends back.
#[derive(Debug, Serialize)]
struct WsResponse {
    /// HTML string to append to the terminal output.
    output: String,
    /// Whether the client should clear the terminal before rendering output.
    clear: bool,
}

/// HTTP upgrade handler — axum calls this when a WebSocket handshake arrives.
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let response = process(&text);

        let json_str: String = serde_json::to_string(&response).unwrap_or_default();
        if socket.send(Message::Text(json_str.into())).await.is_err() {
            break;
        }
    }
}

fn process(raw: &str) -> WsResponse {
    let line = match serde_json::from_str::<WsRequest>(raw) {
        Ok(r) => r.line,
        Err(_) => {
            return WsResponse {
                output: r#"<span class="err">protocol error</span>"#.to_string(),
                clear: false,
            }
        }
    };

    let trimmed = line.trim();
    let mut parts = trimmed.splitn(64, ' ');
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    let raw_output = commands::dispatch(cmd, &args);

    if raw_output == "__CLEAR__" {
        return WsResponse {
            output: String::new(),
            clear: true,
        };
    }

    let output = if raw_output == "__DATE__" {
        format!(
            r#"<span class="out">{}</span>"#,
            Local::now().format("%a %b %e %T %Z %Y")
        )
    } else {
        raw_output
    };

    WsResponse { output, clear: false }
}


