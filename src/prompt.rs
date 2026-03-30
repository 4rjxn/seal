use std::env;

use nix::unistd::{User, gethostname, getuid};
use regex::Regex;

pub fn set_prompt() -> String {
    let user = User::from_uid(getuid()).unwrap().unwrap();
    let current_dir = env::current_dir().unwrap();
    let host = gethostname().unwrap();
    let re = Regex::new(r"/\w+/\w+").unwrap();
    let dir = re.replace(current_dir.to_str().unwrap(), "~");
    let prompt = format!(
        "\n---- {} ᛟ {} ᛯ {} ᚠ\n ᛝ ",
        user.name,
        host.to_str().unwrap(),
        dir
    );
    prompt
}
