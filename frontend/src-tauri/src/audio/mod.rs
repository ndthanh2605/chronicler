//! Audio capture, mixing, and WAV persistence for Chronicler (story S03).
//!
//! Layering (see `docs/stories/phase-1-audio-capture/S03-mic-loopback-mixed-wav.design.md`):
//!
//! - Pure, cross-platform, unit-tested modules: [`meeting_id`], [`wav_writer`],
//!   [`mixer`], and the metering math in [`vu`]. These build and test on any
//!   host (Linux CI included) and never reference the `windows` crate.
//! - Native WASAPI capture ([`wasapi`]) is Windows-only and gated behind
//!   `#[cfg(windows)]`. The capture layer converts device packets into the
//!   platform-neutral [`CapturedFrame`] at the boundary, so the mixer below it
//!   stays platform-agnostic and testable.

// The pure modules are consumed by the Windows-only capture/controller path.
// On non-Windows hosts (Linux CI/dev) only the unit tests reference them, so
// silence dead-code warnings there while keeping them live on the real target.
#![cfg_attr(not(windows), allow(dead_code))]

use std::path::{Path, PathBuf};

pub mod meeting_id;
pub mod mixer;
pub mod vu;
pub mod wav_writer;

#[cfg(windows)]
pub mod wasapi;

/// faster-whisper (Phase 2) consumes 16 kHz mono PCM, so that is the mixer's
/// fixed output format regardless of each device's native capture rate.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;
pub const TARGET_CHANNELS: u16 = 1;

/// Which physical stream a captured frame came from. The mixer keeps the two
/// streams independently aligned and `vu` reports an independent level per id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamId {
    /// Default Windows microphone (capture endpoint).
    Mic,
    /// Default render endpoint captured via WASAPI loopback (system audio).
    Loopback,
}

/// A block of audio captured from one device, in that device's native format.
///
/// This is the **platform-neutral seam** between the native capture layer and
/// the pure mixer: the WASAPI code (Windows-only) builds these from device
/// packets, and unit tests construct them directly. No `windows` type appears
/// here, so the mixer compiles and is tested on any host.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Which stream produced this frame.
    pub stream: StreamId,
    /// Capture timestamp in nanoseconds on a monotonic timeline shared by both
    /// streams (derived from the WASAPI device clock / QPC on Windows; supplied
    /// synthetically in tests). Used for timestamp-based alignment, not sample
    /// counting, so the two independently-clocked devices stay within ±50 ms.
    pub capture_ts_ns: i64,
    /// Native sample rate of the source device (commonly 44_100 or 48_000).
    pub sample_rate: u32,
    /// Native channel count of the source device (commonly 1 or 2).
    pub channels: u16,
    /// Interleaved native PCM samples as f32 in [-1.0, 1.0].
    pub samples: Vec<f32>,
}

/// Resolve `%APPDATA%\Chronicler\audio\` (created if missing). The WAV for each
/// recording is written here as `<meeting-id>.wav` (design §1, architecture
/// Audio File Convention).
pub fn audio_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("cannot resolve app data dir: {e}"))?
        .join("audio");
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {dir:?}: {e}"))?;
    Ok(dir)
}

/// Scan `dir` for WAVs left unfinalized by a forced kill and repair their
/// headers in place (design §5.3 / AC6). Returns how many files were repaired.
/// Run once at app startup, before any new recording.
pub fn repair_partials(dir: &Path) -> std::io::Result<usize> {
    let mut repaired = 0;
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e),
    };
    for entry in entries {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("wav") {
            continue;
        }
        let mut file = std::fs::OpenOptions::new().read(true).write(true).open(&path)?;
        if wav_writer::repair(&mut file)? {
            repaired += 1;
        }
    }
    Ok(repaired)
}

/// Owns a live recording: the capture/mixer/writer threads on Windows, and the
/// `<meeting-id>` whose WAV is being written. Mirrors S02's `Mutex<Option<…>>`
/// + deterministic teardown discipline (design §6).
///
/// The type and its methods compile on every platform so the Tauri command
/// handlers register identically; only the bodies are gated. On non-Windows
/// hosts `start` returns a clear error (capture is Windows-only — ADR-0005).
pub struct AudioController {
    meeting_id: String,
    #[cfg(windows)]
    session: wasapi::CaptureSession,
}

impl AudioController {
    /// Open both devices, begin streaming to `<meeting-id>.wav`, and spawn the
    /// capture/mixer/writer threads.
    pub fn start(app: tauri::AppHandle) -> Result<Self, String> {
        let meeting_id = meeting_id::generate();
        #[cfg(windows)]
        {
            let dir = audio_dir(&app)?;
            let wav_path = dir.join(format!("{meeting_id}.wav"));
            let session = wasapi::CaptureSession::start(app, wav_path)?;
            Ok(Self {
                meeting_id,
                session,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = (app, &meeting_id);
            Err("Audio capture is only supported on Windows.".to_string())
        }
    }

    /// The id used for this recording's WAV filename.
    pub fn meeting_id(&self) -> &str {
        &self.meeting_id
    }

    /// Signal the threads, join them, drain the mixer, finalize the WAV header,
    /// and release every WASAPI handle (design §6 — "no orphan audio handles").
    pub fn stop(self) -> Result<(), String> {
        #[cfg(windows)]
        {
            self.session.stop()
        }
        #[cfg(not(windows))]
        {
            Ok(())
        }
    }
}
