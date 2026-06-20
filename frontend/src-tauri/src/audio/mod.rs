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

pub mod meeting_id;
pub mod mixer;
pub mod vu;
pub mod wav_writer;

// `wasapi` (native capture) + `AudioController` land in the next commit; the
// pure, cross-platform core (above) is committed first with its unit tests.

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
