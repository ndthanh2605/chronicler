//! VU metering: per-stream level math + the event payload the React bars read.
//!
//! The pure level math lives here and is unit-tested cross-platform; the
//! mixer computes per-stream peaks before downmix (so mic and loopback report
//! independently — AC3) and the Windows controller emits [`VuLevels`] as a
//! Tauri event at ≥10 Hz. Levels reach the UI via the event channel, never by
//! polling the WAV.

use serde::Serialize;

/// Peak (maximum absolute) level of a buffer, in `[0.0, 1.0]`.
pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()))
}

/// Root-mean-square level of a buffer, in `[0.0, 1.0]`. Empty buffer → 0.
///
/// Currently only exercised by unit tests; retained alongside [`peak`] as the
/// RMS option for the VU meter (peak is the live default — AC3).
#[cfg_attr(windows, allow(dead_code))]
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|x| x * x).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Independent per-stream levels emitted to the React UI as a Tauri event (AC3).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct VuLevels {
    pub mic: f32,
    pub loopback: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_returns_max_absolute_sample() {
        assert!((peak(&[0.3, -0.6, 0.1]) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn peak_of_silence_is_zero() {
        assert_eq!(peak(&[0.0; 8]), 0.0);
        assert_eq!(peak(&[]), 0.0);
    }

    #[test]
    fn rms_of_constant_signal_equals_its_magnitude() {
        assert!((rms(&[0.5, -0.5, 0.5, -0.5]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn rms_of_silence_is_zero() {
        assert_eq!(rms(&[0.0; 8]), 0.0);
        assert_eq!(rms(&[]), 0.0);
    }

    #[test]
    fn levels_serialize_with_mic_and_loopback_fields() {
        let json = serde_json::to_string(&VuLevels {
            mic: 0.5,
            loopback: 0.25,
        })
        .unwrap();
        assert!(json.contains("\"mic\""), "{json}");
        assert!(json.contains("\"loopback\""), "{json}");
    }
}
