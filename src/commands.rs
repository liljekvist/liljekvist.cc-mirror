pub fn dispatch(cmd: &str, args: &[&str], secret: &str) -> String {
    match cmd {
        "help"   => cmd_help(),
        "whoami" => cmd_whoami(),
        "uname"  => cmd_uname(args),
        "uptime" => cmd_uptime(),
        "echo"   => cmd_echo(args, secret),
        "clear"  => "__CLEAR__".to_string(),
        "date"   => cmd_date(),
        "links"  => cmd_links(),
        "ls"     => cmd_ls(),
        "cat"    => cmd_cat(args),
        "pwd"    => cmd_pwd(),
        "cd"     => cmd_cd(args),
        ""       => String::new(),
        other    => format!(
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
  <span class="cmd">ls</span>       — list directory contents
  <span class="cmd">cat</span>      — print file contents
  <span class="cmd">pwd</span>      — print working directory
  <span class="cmd">cd</span>       — change directory
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

fn cmd_echo(args: &[&str], secret: &str) -> String {
    // Expand $SECRET → the actual secret value; all other $VAR stay as-is.
    let expanded: Vec<String> = args
        .iter()
        .map(|&tok| {
            if tok == "$SECRET" {
                if secret.is_empty() {
                    r#"<span class="err">$SECRET: unbound variable</span>"#.to_string()
                } else {
                    html_escape(secret)
                }
            } else {
                html_escape(tok)
            }
        })
        .collect();
    format!(r#"<span class="out">{}</span>"#, expanded.join(" "))
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

// Virtual filesystem visible to the user.
const FILES: &[(&str, &str)] = &[
    ("welcome.txt",  "__WELCOME__"),   // placeholder — rendered by the WS handler
    ("shortcuts.txt", r#"<a class="lnk" href="https://git.liljekvist.cc"      target="_blank">git.liljekvist.cc</a><span class="dim"> | </span><a class="lnk" href="https://panel.liljekvist.cc"    target="_blank">panel.liljekvist.cc</a><span class="dim"> | </span><a class="lnk" href="https://docker.liljekvist.cc"   target="_blank">docker.liljekvist.cc</a><span class="dim"> | </span><a class="lnk" href="https://registry.liljekvist.cc" target="_blank">registry.liljekvist.cc</a>"#),
];

fn cmd_ls() -> String {
    let entries: Vec<String> = FILES
        .iter()
        .map(|(name, _)| format!(r#"<span class="out">{}</span>"#, name))
        .collect();
    entries.join("  ")
}

fn cmd_cat(args: &[&str]) -> String {
    if args.is_empty() {
        return r#"<span class="err">cat: missing operand</span>"#.to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    for &arg in args {
        // Strip && chaining tokens if the dispatcher ever passes them through.
        if arg == "&&" { continue; }
        match FILES.iter().find(|(name, _)| *name == arg) {
            Some((_, content)) => parts.push(content.to_string()),
            None => parts.push(format!(
                r#"<span class="err">cat: {}: No such file or directory</span>"#,
                html_escape(arg)
            )),
        }
    }
    parts.join("\n")
}

fn cmd_pwd() -> String {
    r#"<span class="out">/home/visitor</span>"#.to_string()
}

fn cmd_cd(args: &[&str]) -> String {
    let target = args.first().copied().unwrap_or("~");
    // Home dir is fine; everything else is "not found".
    match target {
        "~" | "" | "/home/visitor" => String::new(),
        other => format!(
            r#"<span class="err">bash: cd: {}: No such file or directory</span>"#,
            html_escape(other)
        ),
    }
}

/// Minimal HTML escaping to prevent XSS from echoed user input.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

