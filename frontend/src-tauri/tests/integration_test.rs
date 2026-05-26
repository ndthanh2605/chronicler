//! Integration test: spawn the PyInstaller binary, curl /health, assert JSON shape.
//! Requires the binary to be present at src-tauri/binaries/chronicler-backend-<triple>[.exe].
//! Run with: cargo test --test integration_test -- --include-ignored
//!
//! The test is marked #[ignore] by default so that `validate:quick` (which runs
//! `cargo test`) does not fail when the binary has not yet been built.

use std::env;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

/// Finds the first built chronicler-backend binary in the binaries/ directory.
fn find_backend_binary() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    std::fs::read_dir(&dir).ok()?.filter_map(|e| {
        let path = e.ok()?.path();
        let name = path.file_name()?.to_string_lossy().into_owned();
        (name.starts_with("chronicler-backend-") && !name.ends_with(".gitkeep"))
            .then_some(path)
    }).next()
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct Guard(Child);

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

#[tokio::test]
#[ignore = "requires PyInstaller binary — run with --include-ignored"]
async fn test_health_endpoint_json_shape() {
    let binary = match find_backend_binary() {
        Some(p) => p,
        None => {
            eprintln!("Binary not found — skipping integration test");
            return;
        }
    };

    let port = free_port();
    let child = Command::new(&binary)
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .expect("failed to spawn backend binary");
    let _guard = Guard(child);

    // Poll until the server is ready (up to 5 s)
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/health");
    let mut ready = false;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if client.get(&url).send().await.is_ok() {
            ready = true;
            break;
        }
    }
    assert!(ready, "backend did not become ready within 5 s");

    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body.get("last_seen_at").is_some(), "missing last_seen_at field");
}
