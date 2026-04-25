use std::fs;
use std::path::Path;

fn main() {
    let env_file = Path::new(".env");
    let version = if env_file.exists() {
        let content = fs::read_to_string(env_file).unwrap_or_default();
        content
            .lines()
            .find(|line| line.starts_with("VERSION="))
            .and_then(|line| line.split('=').nth(1))
            .map_or_else(
                || std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string()),
                |v| v.trim().to_string(),
            )
    } else {
        std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string())
    };

    println!("cargo:rustc-env=APP_VERSION={version}");
    println!("cargo:rerun-if-changed=.env");
}
