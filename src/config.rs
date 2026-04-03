/// Application configuration, resolved at compile time based on build profile.
///
/// - `cargo build` / `cargo run`           → dev profile  (`debug_assertions` ON)
/// - `cargo build --release` / `cargo run --release` → release profile (`debug_assertions` OFF)
#[derive(Debug, Clone)]
pub struct Config {
    /// Address the HTTP server binds to.
    pub bind_addr: &'static str,
    /// Glob pattern used to discover Tera templates.
    pub template_glob: &'static str,
    /// Directory containing ASCII-art `.txt` files.
    pub ascii_art_dir: &'static str,
    /// Whether to reload templates from disk on every request.
    pub template_autoreload: bool,
    /// Enable verbose/debug logging.
    pub debug_logging: bool,
}

/// Development configuration — hot-reload templates, verbose output, localhost only.
#[cfg(debug_assertions)]
pub fn get() -> Config {
    Config {
        bind_addr: "127.0.0.1:3000",
        template_glob: "templates/**/*",
        ascii_art_dir: "ascii_art",
        template_autoreload: true,
        debug_logging: true,
    }
}

/// Production configuration — no template reload, bind all interfaces, quiet logging.
#[cfg(not(debug_assertions))]
pub fn get() -> Config {
    Config {
        bind_addr: "0.0.0.0:3000",
        template_glob: "templates/**/*",
        ascii_art_dir: "ascii_art",
        template_autoreload: false,
        debug_logging: false,
    }
}

