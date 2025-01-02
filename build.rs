use std::env;
use std::fs;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not set");
    let cache_dir = format!("{}/gpt_responses", out_dir);

    if !fs::exists(&cache_dir).expect("Failed to check if cache directory exists") {
        fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");
    }
}
