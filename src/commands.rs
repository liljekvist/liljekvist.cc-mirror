//! Built-in terminal commands executed server-side.
//!
//! Each function receives the argument tokens (everything after the command name)
//! and returns a `String` that will be sent back to the client as HTML.

/// Dispatch a parsed command to the appropriate handler.
/// Returns an HTML string to display in the terminal output.
pub fn dispatch(cmd: &str, args: &[&str]) -> String {
    match cmd {
        "help" => cmd_help(),
        "whoami" => cmd_whoami(),
        "uname" => cmd_uname(args),
        "uptime" => cmd_uptime(),
        "echo" => cmd_echo(args),
        "clear" => "__CLEAR__".to_string(),
        "date" => cmd_date(),
        "links" => cmd_links(),
        "" => String::new(),
        other => format!(
            r#"<span class="err">bash: {}: command not found</span>"#,
            html_escape(other)
        ),
    }
}

fn cmd_help() -> String {
    r#"<span class="kw">Available commands:</span>
  <span class="cmd">help</span>     — show this help
  <span class="cmd">whoami</span>   — current user
  <span class="cmd">uname</span>    — system info  (<span class="arg">-a</span> for all)
  <span class="cmd">uptime</span>   — server uptime
  <span class="cmd">date</span>     — current server date/time
  <span class="cmd">echo</span>     — print arguments
  <span class="cmd">links</span>    — show site links
  <span class="cmd">clear</span>    — clear terminal"#
        .to_string()
}

fn cmd_whoami() -> String {
    r#"<span class="out">visitor</span>"#.to_string()
}

fn cmd_uname(args: &[&str]) -> String {
    let all = args.contains(&"-a");
    if all {
        format!(
            r#"<span class="out">Linux liljekvist.cc {} {} GNU/Linux</span>"#,
            env!("CARGO_PKG_VERSION"),
            std::env::consts::ARCH
        )
    } else {
        r#"<span class="out">Linux</span>"#.to_string()
    }
}

fn cmd_uptime() -> String {
    // Static message — real uptime would require platform-specific calls.
    r#"<span class="out">up and running</span>"#.to_string()
}

fn cmd_echo(args: &[&str]) -> String {
    format!(
        r#"<span class="out">{}</span>"#,
        html_escape(&args.join(" "))
    )
}

fn cmd_date() -> String {
    // The actual timestamp is injected by the WS handler which has access to
    // std::time — we return a placeholder that the handler replaces.
    "__DATE__".to_string()
}

fn cmd_links() -> String {
    r#"<span class="kw">Links:</span>
  <a href="https://github.com/liljekvist" target="_blank" class="lnk">github.com/liljekvist</a>
  <a href="https://git.liljekvist.cc" class="lnk">git.liljekvist.cc</a>"#
        .to_string()
}

/// Minimal HTML escaping to prevent XSS from echoed user input.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

