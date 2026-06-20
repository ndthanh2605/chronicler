use std::net::TcpListener;
use std::sync::{Mutex, OnceLock};

use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

mod audio;

static BACKEND_PORT: OnceLock<u16> = OnceLock::new();
static BACKEND_CHILD: Mutex<Option<CommandChild>> = Mutex::new(None);

/// Bind to port 0, let the OS assign a free port, then immediately release the binding.
/// The caller must pass the returned port to FastAPI before another process claims it.
fn pick_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind ephemeral port")
        .local_addr()
        .unwrap()
        .port()
}

#[tauri::command]
fn get_backend_port() -> u16 {
    *BACKEND_PORT.get().expect("backend port not initialized")
}

/// Kill the sidecar and any children it spawned (PyInstaller's onefile
/// bootloader extracts and launches the real interpreter as a child
/// process, which `CommandChild::kill()` alone leaves running as an
/// orphan). On Windows, `taskkill /T` kills the whole process tree.
#[cfg(windows)]
fn kill_backend(child: CommandChild) {
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/T", "/PID", &child.pid().to_string()])
        .status();
}

#[cfg(not(windows))]
fn kill_backend(child: CommandChild) {
    let _ = child.kill();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let port = pick_free_port();
            BACKEND_PORT.set(port).expect("port already set");

            // Spawn failures must not crash the app (AC6) — the React UI
            // detects a dead backend via the failed `/health` fetch and
            // shows "Backend unavailable" instead.
            match app.shell().sidecar("binaries/chronicler-backend") {
                Ok(cmd) => match cmd.args(["--port", &port.to_string()]).spawn() {
                    Ok((_, child)) => {
                        *BACKEND_CHILD.lock().unwrap() = Some(child);
                    }
                    Err(err) => {
                        eprintln!("failed to spawn backend sidecar: {err}");
                    }
                },
                Err(err) => {
                    eprintln!("backend sidecar binary not found: {err}");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_backend_port])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if let Ok(mut guard) = BACKEND_CHILD.lock() {
                    if let Some(child) = guard.take() {
                        kill_backend(child);
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_free_port_returns_nonzero() {
        assert!(pick_free_port() > 0);
    }
}
