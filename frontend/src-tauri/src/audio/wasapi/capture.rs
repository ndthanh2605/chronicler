//! Default-microphone capture loop (WASAPI shared-mode, polling drain).

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::audio::mixer::Mixer;
use crate::audio::StreamId;

/// Capture the default Windows microphone, pushing timestamped frames to the
/// mixer until `stop` is set. See `super::run_capture` for the shared loop.
pub fn run(mixer: Arc<Mutex<Mixer>>, stop: Arc<AtomicBool>) -> windows::core::Result<()> {
    super::run_capture(StreamId::Mic, false, mixer, stop)
}
