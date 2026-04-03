use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::commands;
use crate::state::AppState;

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
pub async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(Ok(msg)) = socket.recv().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let response = process(&text, &state);

        let json_str: String = serde_json::to_string(&response).unwrap_or_default();
        if socket.send(Message::Text(json_str.into())).await.is_err() {
            break;
        }
    }
}

fn process(raw: &str, state: &AppState) -> WsResponse {
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

    let raw_output = commands::dispatch(cmd, &args, &state.secret);

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
    } else if raw_output.contains("__WELCOME__") {
        // Replace the placeholder with a randomly chosen ASCII banner.
        let banner = crate::ascii_art::random(&state.ascii_styles);
        raw_output.replace(
            "__WELCOME__",
            &format!(r#"<pre class="ascii-banner">{}</pre>"#, banner),
        )
    } else {
        raw_output
    };

    WsResponse { output, clear: false }
}


