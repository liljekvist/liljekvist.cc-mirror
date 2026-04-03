# liljekvist.cc

Personal start-page for [liljekvist.cc](https://liljekvist.cc) — a bash-terminal-styled single page served by a Rust/Axum backend with a live WebSocket command interface.

## Stack

| Layer | Technology |
|-------|-----------|
| Server | [Axum](https://github.com/tokio-rs/axum) + Tokio |
| Templates | [Tera](https://keats.github.io/tera/) via axum-template |
| Static files | tower-http `ServeDir` |
| Real-time | WebSocket (`/ws`) |
| Deploy | Docker + GitLab CI → nginx reverse proxy |

## Project layout

```
src/
  main.rs          # router + server startup
  config.rs        # dev / release configuration
  state.rs         # shared AppState (engine, ascii styles, secret)
  commands.rs      # built-in terminal commands (help, ls, cat, pwd, …)
  ascii_art.rs     # loads ascii_art/*.txt at startup, picks one at random
  handlers/
    mod.rs         # GET / → renders index.html
    ws.rs          # WebSocket handler
templates/
  base.html        # layout, CSS
  index.html       # terminal UI + JS client
ascii_art/         # one .txt file per ASCII banner style
assets/            # favicons + web manifest
```

## Running locally

```bash
# dev  (hot-template-reload, binds 127.0.0.1:3000)
cargo run

# release  (binds 0.0.0.0:3000)
cargo run --release
```

## Adding a new ASCII art style

Drop any `.txt` file into `ascii_art/` and restart the server — it will be picked up automatically and included in the random rotation.

## Deployment

The GitLab CI pipeline builds the release binary and packages it into a Docker image.  
nginx proxies `liljekvist.cc` → the container on port 3000, upgrading WebSocket connections at `/ws`.
