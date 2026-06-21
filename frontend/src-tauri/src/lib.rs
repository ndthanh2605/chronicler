use std::net::TcpListener;
use std::sync::{Mutex, OnceLock};

use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

mod audio;

static BACKEND_PORT: OnceLock<u16> = OnceLock::new();
static BACKEND_CHILD: Mutex<Option<CommandChild>> = Mutex::new(None);

/// The live recording, if any. Mirrors the S02 sidecar-lifecycle pattern: a
/// module-level `Mutex<Option<…>>` started/stopped from IPC and torn down on
/// the window `Destroyed` event (design §6).
static AUDIO: Mutex<Option<audio::AudioController>> = Mutex::new(None);

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

/// Open the mic + loopback, begin streaming the mixed WAV, and return the
/// `<meeting-id>`. Non-fatal: a device-open failure returns a clear error to
/// the UI rather than crashing the app (design §6).
#[tauri::command]
fn start_recording(app: tauri::AppHandle) -> Result<String, String> {
    let mut guard = AUDIO.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Err("already recording".to_string());
    }
    let controller = audio::AudioController::start(app)?;
    let id = controller.meeting_id().to_string();
    *guard = Some(controller);
    Ok(id)
}

/// Stop the recording: signal threads, finalize the WAV header, release all
/// WASAPI handles. Idempotent — a no-op if nothing is recording.
#[tauri::command]
fn stop_recording() -> Result<(), String> {
    let controller = {
        let mut guard = AUDIO.lock().map_err(|e| e.to_string())?;
        guard.take()
    };
    match controller {
        Some(c) => c.stop(),
        None => Ok(()),
    }
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

            // Auto-repair any WAV left unfinalized by a previous forced kill
            // (design §5.3 / AC6) before a new recording can start.
            if let Ok(dir) = audio::audio_dir(app.handle()) {
                match audio::repair_partials(&dir) {
                    Ok(n) if n > 0 => eprintln!("repaired {n} unfinalized recording(s)"),
                    Ok(_) => {}
                    Err(err) => eprintln!("partial-recovery scan failed: {err}"),
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_backend_port,
            start_recording,
            stop_recording
        ])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // Stop any in-flight recording first so the WAV header is
                // finalized and no WASAPI handle is orphaned.
                if let Ok(mut guard) = AUDIO.lock() {
                    if let Some(controller) = guard.take() {
                        let _ = controller.stop();
                    }
                }
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
