//! Ring-buffer mixer: two timestamp-aligned streams → 16 kHz mono PCM.
//!
//! Both capture streams run on independent hardware clocks, so the mixer aligns
//! by **capture timestamp**, not by counting samples (design §5.1). Each frame
//! is downmixed to mono, resampled to 16 kHz, and added into a common output
//! timeline at the slot its timestamp maps to. Unfilled slots stay zero, so a
//! silent or absent stream contributes silence for free (AC5) — no stall, no
//! explicit silence frames. Draining keeps only a ~50 ms alignment window
//! buffered, so memory is bounded regardless of run length (AC4).

use std::collections::VecDeque;

use super::{CapturedFrame, StreamId, TARGET_SAMPLE_RATE};

/// Output samples (~50 ms at 16 kHz) held back from draining so a late packet
/// from the other stream can still land in its timestamp slot before emission.
const ALIGN_WINDOW_SAMPLES: u64 = 800;

fn stream_index(s: StreamId) -> usize {
    match s {
        StreamId::Mic => 0,
        StreamId::Loopback => 1,
    }
}

/// Average interleaved channels down to mono.
fn downmix_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    let c = channels.max(1) as usize;
    if c == 1 {
        return samples.to_vec();
    }
    samples
        .chunks(c)
        .map(|frame| frame.iter().sum::<f32>() / c as f32)
        .collect()
}

/// Linear-interpolation resampler (mono). Adequate for 16 kHz speech mixing in
/// Phase 1; a higher-quality polyphase resampler (`rubato`) can replace this
/// later without touching the alignment logic (ADR-0005 left the crate open).
fn resample_linear(input: &[f32], in_rate: u32, out_rate: u32) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }
    if in_rate == out_rate {
        return input.to_vec();
    }
    let ratio = out_rate as f64 / in_rate as f64;
    let out_len = (input.len() as f64 * ratio).round() as usize;
    let last = input.len() - 1;
    (0..out_len)
        .map(|i| {
            let src = i as f64 / ratio;
            let i0 = (src.floor() as usize).min(last);
            let i1 = (i0 + 1).min(last);
            let frac = (src - i0 as f64) as f32;
            input[i0] + (input[i1] - input[i0]) * frac
        })
        .collect()
}

fn to_i16(x: f32) -> i16 {
    (x.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Two-stream ring-buffer mixer producing 16 kHz mono PCM.
pub struct Mixer {
    /// Timestamp (ns) mapped to output index 0; set by the first frame seen.
    anchor_ns: Option<i64>,
    /// Output index of `mixbuf`'s front element.
    base_index: u64,
    /// Mixed f32 sums from `base_index` forward; unfilled slots are silence.
    mixbuf: VecDeque<f32>,
    /// Exclusive output index each stream [mic, loopback] has delivered up to.
    filled: [u64; 2],
    /// Peak level of each stream's most recent frame (pre-downmix), for VU.
    last_peak: [f32; 2],
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            anchor_ns: None,
            base_index: 0,
            mixbuf: VecDeque::new(),
            filled: [0; 2],
            last_peak: [0.0; 2],
        }
    }

    /// Feed one captured frame (native rate/channels). Downmixed, resampled to
    /// 16 kHz, and placed on the common timeline by its capture timestamp.
    pub fn push(&mut self, frame: CapturedFrame) {
        let si = stream_index(frame.stream);
        let mono = downmix_mono(&frame.samples, frame.channels);
        self.last_peak[si] = super::vu::peak(&mono);

        let res = resample_linear(&mono, frame.sample_rate, TARGET_SAMPLE_RATE);
        if res.is_empty() {
            return;
        }

        let anchor = *self.anchor_ns.get_or_insert(frame.capture_ts_ns);
        let ns_per_sample = 1_000_000_000f64 / TARGET_SAMPLE_RATE as f64;
        let mapped = (((frame.capture_ts_ns - anchor) as f64) / ns_per_sample).round();
        // Clamp to the live window: never before what has already been drained.
        let start_index = (mapped.max(0.0) as u64).max(self.base_index);

        let end_index = start_index + res.len() as u64;
        let needed = end_index.saturating_sub(self.base_index);
        while (self.mixbuf.len() as u64) < needed {
            self.mixbuf.push_back(0.0);
        }
        for (k, &s) in res.iter().enumerate() {
            let idx = start_index + k as u64;
            if idx < self.base_index {
                continue; // slot already drained; drop the late sample
            }
            let off = (idx - self.base_index) as usize;
            self.mixbuf[off] += s;
        }
        self.filled[si] = self.filled[si].max(end_index);
    }

    fn max_filled(&self) -> u64 {
        self.filled[0].max(self.filled[1])
    }

    /// Emit mixed output that is safe to commit (older than the alignment
    /// window). Keeps the buffer bounded during a live recording.
    pub fn drain_ready(&mut self) -> Vec<i16> {
        let ready = self.max_filled().saturating_sub(ALIGN_WINDOW_SAMPLES);
        self.drain_until(ready)
    }

    /// Emit everything remaining (called on Stop).
    pub fn flush(&mut self) -> Vec<i16> {
        let end = self.max_filled();
        self.drain_until(end)
    }

    fn drain_until(&mut self, end_index: u64) -> Vec<i16> {
        let mut out = Vec::new();
        while self.base_index < end_index {
            let sample = self.mixbuf.pop_front().unwrap_or(0.0);
            out.push(to_i16(sample));
            self.base_index += 1;
        }
        out
    }

    /// Peak level [mic, loopback] of each stream's most recent frame, in
    /// [0.0, 1.0] — the input the VU meters render (AC3).
    pub fn last_levels(&self) -> [f32; 2] {
        self.last_peak
    }

    #[cfg(test)]
    fn buffered_len(&self) -> usize {
        self.mixbuf.len()
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{CapturedFrame, StreamId};

    /// i16 value a normalized f32 sample maps to (matches the mixer's scaling).
    fn scaled(x: f32) -> i16 {
        (x.clamp(-1.0, 1.0) * 32767.0).round() as i16
    }

    fn frame(stream: StreamId, ts_ns: i64, rate: u32, ch: u16, samples: Vec<f32>) -> CapturedFrame {
        CapturedFrame {
            stream,
            capture_ts_ns: ts_ns,
            sample_rate: rate,
            channels: ch,
            samples,
        }
    }

    #[test]
    fn mono_16k_passes_through_with_scaling() {
        let mut m = Mixer::new();
        m.push(frame(StreamId::Mic, 0, 16_000, 1, vec![0.5; 4]));
        let out = m.flush();
        assert_eq!(out.len(), 4);
        for s in out {
            assert!((s - scaled(0.5)).abs() <= 1, "got {s}");
        }
    }

    #[test]
    fn sums_two_aligned_streams() {
        let mut m = Mixer::new();
        m.push(frame(StreamId::Mic, 0, 16_000, 1, vec![0.5, 0.5]));
        m.push(frame(StreamId::Loopback, 0, 16_000, 1, vec![0.25, 0.25]));
        let out = m.flush();
        assert_eq!(out.len(), 2);
        for s in out {
            assert!((s - scaled(0.75)).abs() <= 1, "expected ~0.75 sum, got {s}");
        }
    }

    #[test]
    fn clamps_summed_overflow() {
        let mut m = Mixer::new();
        m.push(frame(StreamId::Mic, 0, 16_000, 1, vec![0.8]));
        m.push(frame(StreamId::Loopback, 0, 16_000, 1, vec![0.8]));
        let out = m.flush();
        assert_eq!(out[0], 32767, "1.6 must clamp to i16::MAX");
    }

    #[test]
    fn aligns_impulse_by_timestamp_not_sample_count() {
        // Mic delivers 0.5 s of silence, then a 0.5 s GAP in delivery, then an
        // impulse stamped at t=1 s. Sample-counting would place the impulse at
        // index 8000 (only 8000 samples delivered); timestamp alignment must
        // place it at index 16000 (t=1 s). This is the ±50 ms sync mechanism.
        let mut m = Mixer::new();
        m.push(frame(StreamId::Mic, 0, 16_000, 1, vec![0.0; 8000]));
        let one_second_ns = 1_000_000_000;
        let mut impulse = vec![0.0; 10];
        impulse[0] = 1.0;
        m.push(frame(StreamId::Mic, one_second_ns, 16_000, 1, impulse));

        let out = m.flush();
        let argmax = out
            .iter()
            .enumerate()
            .max_by_key(|(_, v)| v.abs())
            .map(|(i, _)| i)
            .unwrap();
        let tolerance = 800; // 50 ms at 16 kHz
        assert!(
            (argmax as i64 - 16_000).abs() <= tolerance,
            "impulse at index {argmax}, expected ~16000 (±50 ms); sample-count bug would give ~8000"
        );
    }

    #[test]
    fn gap_in_one_stream_is_silence_and_does_not_stall() {
        // Mic continuous over 2 s; loopback present only in the first and last
        // 0.5 s with a 1 s gap. Output must span the full 2 s (no stall), with
        // the gap region carrying mic-only level (loopback contributes silence).
        let mut m = Mixer::new();
        m.push(frame(StreamId::Mic, 0, 16_000, 1, vec![0.5; 32_000]));
        m.push(frame(StreamId::Loopback, 0, 16_000, 1, vec![0.5; 8_000]));
        let gap_end_ns = 1_500_000_000; // 1.5 s
        m.push(frame(StreamId::Loopback, gap_end_ns, 16_000, 1, vec![0.5; 8_000]));

        let out = m.flush();
        assert_eq!(out.len(), 32_000, "must cover full mic timeline, no stall");
        assert!((out[0] - scaled(1.0)).abs() <= 1, "overlap region = both");
        assert!(
            (out[16_000] - scaled(0.5)).abs() <= 1,
            "gap region = mic only (loopback silence)"
        );
    }

    #[test]
    fn drain_ready_keeps_buffer_bounded() {
        let mut m = Mixer::new();
        let chunk = 1600usize; // 100 ms at 16 kHz
        for i in 0..200i64 {
            m.push(frame(StreamId::Mic, i * 100_000_000, 16_000, 1, vec![0.1; chunk]));
            m.push(frame(StreamId::Loopback, i * 100_000_000, 16_000, 1, vec![0.1; chunk]));
            let _ = m.drain_ready();
            assert!(
                m.buffered_len() < 8000,
                "buffer grew unbounded: {} at iter {i}",
                m.buffered_len()
            );
        }
    }

    #[test]
    fn downmixes_stereo_to_mono() {
        let mut m = Mixer::new();
        // Two interleaved stereo frames: (1.0,-1.0)->0.0 and (0.5,0.5)->0.5.
        m.push(frame(StreamId::Mic, 0, 16_000, 2, vec![1.0, -1.0, 0.5, 0.5]));
        let out = m.flush();
        assert_eq!(out.len(), 2);
        assert!((out[0] - scaled(0.0)).abs() <= 1);
        assert!((out[1] - scaled(0.5)).abs() <= 1);
    }

    #[test]
    fn resamples_native_rate_to_16k() {
        let mut m = Mixer::new();
        // 32 samples at 32 kHz (1 ms) should resample to ~16 samples at 16 kHz.
        m.push(frame(StreamId::Mic, 0, 32_000, 1, vec![0.0; 32]));
        let out = m.flush();
        assert!(
            (out.len() as i64 - 16).abs() <= 1,
            "expected ~16 samples, got {}",
            out.len()
        );
    }

    #[test]
    fn last_levels_reports_independent_per_stream_peak() {
        let mut m = Mixer::new();
        m.push(frame(StreamId::Mic, 0, 16_000, 1, vec![0.3, -0.6]));
        m.push(frame(StreamId::Loopback, 0, 16_000, 1, vec![0.2, 0.1]));
        let levels = m.last_levels();
        assert!((levels[0] - 0.6).abs() < 0.01, "mic peak {}", levels[0]);
        assert!((levels[1] - 0.2).abs() < 0.01, "loopback peak {}", levels[1]);
    }
}
