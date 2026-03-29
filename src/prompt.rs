use std::env;

pub fn set_prompt() -> String {
    let dir = env::current_dir().unwrap();
    let dir = dir.to_str().unwrap().replacen("/home/arjun", "~", 1);
    let prompt = format!("\n---- parzival ᛟ oasis ᛯ {} ᚠ\n ᛝ ", dir);
    prompt
}
