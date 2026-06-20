//! System-audio capture loop: default **render** device with the WASAPI
//! loopback flag (`AUDCLNT_STREAMFLAGS_LOOPBACK`).

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::audio::mixer::Mixer;
use crate::audio::StreamId;

/// Capture the default render device's output via loopback, pushing timestamped
/// frames to the mixer until `stop` is set. When nothing is playing the render
/// device yields no packets, so the loop never blocks and the mixer fills
/// silence (AC5). See `super::run_capture` for the shared loop.
pub fn run(mixer: Arc<Mutex<Mixer>>, stop: Arc<AtomicBool>) -> windows::core::Result<()> {
    super::run_capture(StreamId::Loopback, true, mixer, stop)
}
