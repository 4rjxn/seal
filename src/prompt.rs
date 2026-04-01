use std::env;

use nix::unistd::{User, gethostname, getuid};

fn red(text: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", text)
}
fn bright_red(text: &str) -> String {
    format!("\x1b[91m{}\x1b[0m", text)
}
pub fn set_prompt() -> String {
    let user = User::from_uid(getuid())
        .ok()
        .flatten()
        .map(|u| u.name)
        .unwrap_or_else(|| "unknown".to_string());
    let current_dir = env::current_dir().unwrap_or_else(|_| env::current_dir().unwrap());
    let host = gethostname()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    let home = env::var("HOME").unwrap_or_default();
    let dir_str = current_dir.to_str().unwrap_or("");
    let dir = if !home.is_empty() && dir_str.starts_with(&home) {
        dir_str.replacen(&home, "~", 1)
    } else {
        dir_str.to_string()
    };

    format!(
        "\n---- {} ᛟ {} ᛯ {} ᚠ\n {} ",
        red(&user),
        host,
        dir,
        bright_red("ᛝ")
    )
}
