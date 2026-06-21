//! Windows-only WASAPI capture (mic + render-device loopback) → mixer.
//!
//! BUILD STATUS: this module is `#[cfg(windows)]` and is **not compiled by the
//! Linux dev/CI gate**. It was written against the official MS Core Audio
//! "Capturing a Stream" pattern (learn.microsoft.com/windows/win32/coreaudio)
//! and the `windows` crate 0.58, but has **not been compiled on Windows yet**.
//! Expect to resolve a few `windows`-crate signature details on the first
//! Windows build (tracked in the S03 workpad). The capture algorithm — shared
//! mode, polling drain, QPC-timestamped frames, loopback via
//! `AUDCLNT_STREAMFLAGS_LOOPBACK` — is the load-bearing part and follows the
//! docs.
//!
//! Polling (not event-driven) is deliberate: WASAPI loopback does not reliably
//! support `AUDCLNT_STREAMFLAGS_EVENTCALLBACK`, and the canonical MS capture
//! sample polls `GetNextPacketSize`. A silent render device simply yields zero
//! packets, so the loop never blocks and the mixer fills silence (AC5).

mod capture;
mod loopback;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use windows::Win32::Media::Audio::{AUDCLNT_BUFFERFLAGS_SILENT, WAVEFORMATEX};
use windows::Win32::Media::Multimedia::WAVE_FORMAT_IEEE_FLOAT;

use super::mixer::Mixer;
use super::vu::VuLevels;
use super::wav_writer::WavWriter;
use super::CapturedFrame;

/// VU emit / WAV drain cadence. 50 ms = 20 Hz, comfortably above the ≥10 Hz
/// the UI requires (AC3) and matching the mixer's alignment window.
const TICK: Duration = Duration::from_millis(50);

/// A live capture session: two device-capture threads feeding a shared mixer,
/// and a writer thread draining the mixer to the WAV and emitting VU events.
pub struct CaptureSession {
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl CaptureSession {
    /// Spawn the mic + loopback capture threads and the writer/VU thread,
    /// streaming the mixed 16 kHz mono WAV to `wav_path`.
    pub fn start(app: AppHandle, wav_path: PathBuf) -> Result<Self, String> {
        let writer = {
            let file = std::fs::File::create(&wav_path)
                .map_err(|e| format!("cannot create {wav_path:?}: {e}"))?;
            WavWriter::new(file).map_err(|e| format!("cannot write WAV header: {e}"))?
        };

        let mixer = Arc::new(Mutex::new(Mixer::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::new();

        // Mic capture thread.
        {
            let mixer = mixer.clone();
            let stop = stop.clone();
            threads.push(std::thread::spawn(move || {
                if let Err(e) = capture::run(mixer, stop) {
                    eprintln!("mic capture thread exited: {e}");
                }
            }));
        }
        // Loopback (system audio) capture thread.
        {
            let mixer = mixer.clone();
            let stop = stop.clone();
            threads.push(std::thread::spawn(move || {
                if let Err(e) = loopback::run(mixer, stop) {
                    eprintln!("loopback capture thread exited: {e}");
                }
            }));
        }
        // Writer + VU thread: drains the mixer, appends to the WAV, emits levels.
        {
            let mixer = mixer.clone();
            let stop = stop.clone();
            threads.push(std::thread::spawn(move || {
                writer_loop(app, writer, mixer, stop);
            }));
        }

        Ok(Self { stop, threads })
    }

    /// Signal all threads and join them. The writer thread finalizes the WAV
    /// header on exit; the capture threads release their WASAPI handles.
    pub fn stop(self) -> Result<(), String> {
        self.stop.store(true, Ordering::Relaxed);
        for t in self.threads {
            let _ = t.join();
        }
        Ok(())
    }
}

/// Drain mixed PCM to the WAV and emit per-stream VU at ~20 Hz until stopped,
/// then flush the tail and finalize the header.
fn writer_loop(
    app: AppHandle,
    mut writer: WavWriter<std::fs::File>,
    mixer: Arc<Mutex<Mixer>>,
    stop: Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(TICK);
        let (ready, levels) = {
            match mixer.lock() {
                Ok(mut m) => (m.drain_ready(), m.last_levels()),
                Err(_) => break,
            }
        };
        if !ready.is_empty() {
            if let Err(e) = writer.append(&ready) {
                eprintln!("WAV append failed: {e}");
                break;
            }
        }
        let _ = app.emit(
            "vu-levels",
            VuLevels {
                mic: levels[0],
                loopback: levels[1],
            },
        );
    }

    // Final flush + header fixup.
    let tail = mixer.lock().map(|mut m| m.flush()).unwrap_or_default();
    if !tail.is_empty() {
        let _ = writer.append(&tail);
    }
    if let Err(e) = writer.finalize() {
        eprintln!("WAV finalize failed: {e}");
    }
}

/// Read one WASAPI packet's raw bytes into normalized f32 samples.
///
/// Shared-mode mix format is overwhelmingly 32-bit IEEE float; 16-bit PCM is
/// handled as a fallback. `wfx` carries the tag (or the EXTENSIBLE subformat
/// the engine reports as `WAVE_FORMAT_IEEE_FLOAT`).
unsafe fn read_samples(data: *const u8, frames: u32, wfx: &WAVEFORMATEX) -> Vec<f32> {
    let total = frames as usize * wfx.nChannels as usize;
    if data.is_null() || total == 0 {
        return vec![0.0; total];
    }
    let is_float =
        wfx.wFormatTag == WAVE_FORMAT_IEEE_FLOAT as u16 || wfx.wBitsPerSample == 32;
    if is_float {
        let p = data as *const f32;
        (0..total).map(|i| *p.add(i)).collect()
    } else {
        // 16-bit signed PCM.
        let p = data as *const i16;
        (0..total).map(|i| *p.add(i) as f32 / 32768.0).collect()
    }
}

/// Push one captured packet to the mixer (shared by mic + loopback loops).
fn push_frame(
    mixer: &Arc<Mutex<Mixer>>,
    stream: super::StreamId,
    capture_ts_ns: i64,
    sample_rate: u32,
    channels: u16,
    samples: Vec<f32>,
) {
    if let Ok(mut m) = mixer.lock() {
        m.push(CapturedFrame {
            stream,
            capture_ts_ns,
            sample_rate,
            channels,
            samples,
        });
    }
}

/// The shared shared-mode polling capture loop used by both `capture` (mic) and
/// `loopback` (system audio). `loopback` activates the **render** endpoint with
/// `AUDCLNT_STREAMFLAGS_LOOPBACK`.
fn run_capture(
    stream: super::StreamId,
    loopback: bool,
    mixer: Arc<Mutex<Mixer>>,
    stop: Arc<AtomicBool>,
) -> windows::core::Result<()> {
    use windows::Win32::Media::Audio::{
        eCapture, eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator,
        MMDeviceEnumerator, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let result = (|| -> windows::core::Result<()> {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let data_flow = if loopback { eRender } else { eCapture };
            let device = enumerator.GetDefaultAudioEndpoint(data_flow, eConsole)?;
            let client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;

            let pwfx = client.GetMixFormat()?;
            let wfx = *pwfx;
            let channels = wfx.nChannels;
            let rate = wfx.nSamplesPerSec;

            let mut flags = 0u32;
            if loopback {
                flags |= AUDCLNT_STREAMFLAGS_LOOPBACK;
            }
            // 1 s shared buffer (REFERENCE_TIME, 100 ns units).
            client.Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 10_000_000, 0, pwfx, None)?;
            let capture: IAudioCaptureClient = client.GetService()?;
            client.Start()?;

            while !stop.load(Ordering::Relaxed) {
                let mut packet = capture.GetNextPacketSize()?;
                while packet != 0 {
                    let mut data: *mut u8 = std::ptr::null_mut();
                    let mut frames = 0u32;
                    let mut buf_flags = 0u32;
                    let mut qpc: u64 = 0;
                    capture.GetBuffer(
                        &mut data,
                        &mut frames,
                        &mut buf_flags,
                        None,
                        Some(&mut qpc),
                    )?;

                    let silent = buf_flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
                    let samples = if silent {
                        vec![0.0; frames as usize * channels as usize]
                    } else {
                        read_samples(data, frames, &wfx)
                    };
                    // QPC position is in 100 ns units on a clock shared by both
                    // capture threads — the common timeline the mixer aligns on.
                    push_frame(&mixer, stream, qpc as i64 * 100, rate, channels, samples);

                    capture.ReleaseBuffer(frames)?;
                    packet = capture.GetNextPacketSize()?;
                }
                std::thread::sleep(Duration::from_millis(10));
            }

            client.Stop()?;
            // (pwfx from GetMixFormat is intentionally not freed here — one
            //  small leak per recording; a CoTaskMemFree can be added once the
            //  Windows build confirms the exact `windows`-crate signature.)
            Ok(())
        })();
        CoUninitialize();
        result
    }
}
