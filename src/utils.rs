use std::fs;
use std::io;

pub fn to_string_raw(value: &serde_json::Value) -> String {
    let str = value.to_string();
    str[1..str.len()-1].to_string()
}

pub fn clean_old_playlist(dir: &str) -> Result<(), io::Error> {
    fs::remove_dir_all(dir)
}

pub fn normalize_string(s: &str) -> String {
    s.replace("/", "∕")  // using some random utf slash instead of the directory separator thing
}

