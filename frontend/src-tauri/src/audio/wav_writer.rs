//! Streaming WAV writer for 16 kHz mono 16-bit PCM with header fixup.
//!
//! Policy (design §5.2 / §5.3): write a 44-byte canonical header up front with
//! **placeholder** size fields, append PCM frames continuously, and back-patch
//! the RIFF + `data` sizes on [`WavWriter::finalize`]. A forced-kill partial
//! still carries the placeholder, so [`repair`] can recompute its sizes from
//! the file length on next launch — no meetings DB required in Phase 1.

use std::io::{self, Read, Seek, SeekFrom, Write};

use super::{TARGET_CHANNELS, TARGET_SAMPLE_RATE};

/// Sentinel written into the RIFF and `data` size fields on start. A file still
/// carrying this in its `data` size was never finalized (see [`repair`]).
pub const PLACEHOLDER_SIZE: u32 = 0xFFFF_FFFF;

const BITS_PER_SAMPLE: u16 = 16;
const HEADER_LEN: u32 = 44;
/// Byte offset of the RIFF chunk-size field.
const RIFF_SIZE_OFFSET: u64 = 4;
/// Byte offset of the `data` chunk-size field.
const DATA_SIZE_OFFSET: u64 = 40;

fn block_align() -> u16 {
    TARGET_CHANNELS * (BITS_PER_SAMPLE / 8)
}

/// Streaming WAV writer. Generic over the sink so unit tests drive it with an
/// in-memory `Cursor<Vec<u8>>` and production uses a `File`.
pub struct WavWriter<W: Write + Seek> {
    inner: W,
    data_bytes: u32,
}

impl<W: Write + Seek> WavWriter<W> {
    /// Create a writer and emit the header with placeholder sizes.
    pub fn new(mut inner: W) -> io::Result<Self> {
        write_header(&mut inner, PLACEHOLDER_SIZE, PLACEHOLDER_SIZE)?;
        Ok(Self {
            inner,
            data_bytes: 0,
        })
    }

    /// Append mono 16-bit PCM samples, streaming straight to the sink.
    pub fn append(&mut self, samples: &[i16]) -> io::Result<()> {
        let mut buf = Vec::with_capacity(samples.len() * 2);
        for s in samples {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        self.inner.write_all(&buf)?;
        self.data_bytes += buf.len() as u32;
        Ok(())
    }

    /// Patch the RIFF + `data` size fields to the real sizes and return the sink.
    pub fn finalize(mut self) -> io::Result<W> {
        self.inner
            .seek(SeekFrom::Start(RIFF_SIZE_OFFSET))?;
        self.inner.write_all(&(36 + self.data_bytes).to_le_bytes())?;
        self.inner.seek(SeekFrom::Start(DATA_SIZE_OFFSET))?;
        self.inner.write_all(&self.data_bytes.to_le_bytes())?;
        self.inner.flush()?;
        Ok(self.inner)
    }

    /// Borrow the underlying sink (used in tests to inspect bytes pre-finalize).
    #[cfg(test)]
    pub fn get_ref(&self) -> &W {
        &self.inner
    }
}

fn write_header<W: Write>(w: &mut W, riff_size: u32, data_size: u32) -> io::Result<()> {
    let byte_rate = TARGET_SAMPLE_RATE * block_align() as u32;
    w.write_all(b"RIFF")?;
    w.write_all(&riff_size.to_le_bytes())?;
    w.write_all(b"WAVE")?;
    w.write_all(b"fmt ")?;
    w.write_all(&16u32.to_le_bytes())?; // PCM fmt chunk size
    w.write_all(&1u16.to_le_bytes())?; // audio format = PCM
    w.write_all(&TARGET_CHANNELS.to_le_bytes())?;
    w.write_all(&TARGET_SAMPLE_RATE.to_le_bytes())?;
    w.write_all(&byte_rate.to_le_bytes())?;
    w.write_all(&block_align().to_le_bytes())?;
    w.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;
    w.write_all(b"data")?;
    w.write_all(&data_size.to_le_bytes())?;
    Ok(())
}

/// Repair an unfinalized WAV stream in place.
///
/// If the `data` size field still carries [`PLACEHOLDER_SIZE`], the file was
/// never finalized: recompute the payload length from the total stream length,
/// rounded **down** to a whole sample frame, and patch the RIFF + `data` size
/// fields. Returns `Ok(true)` if it repaired, `Ok(false)` if already finalized.
pub fn repair<S: Read + Write + Seek>(stream: &mut S) -> io::Result<bool> {
    let total = stream.seek(SeekFrom::End(0))?;
    if total < HEADER_LEN as u64 {
        return Ok(false);
    }

    stream.seek(SeekFrom::Start(DATA_SIZE_OFFSET))?;
    let mut field = [0u8; 4];
    stream.read_exact(&mut field)?;
    if u32::from_le_bytes(field) != PLACEHOLDER_SIZE {
        return Ok(false);
    }

    let payload = total - HEADER_LEN as u64;
    let align = block_align() as u64;
    let data_bytes = (payload - payload % align) as u32;

    stream.seek(SeekFrom::Start(RIFF_SIZE_OFFSET))?;
    stream.write_all(&(36 + data_bytes).to_le_bytes())?;
    stream.seek(SeekFrom::Start(DATA_SIZE_OFFSET))?;
    stream.write_all(&data_bytes.to_le_bytes())?;
    stream.flush()?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn u32_le(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }
    fn u16_le(bytes: &[u8], at: usize) -> u16 {
        u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
    }

    #[test]
    fn header_describes_16khz_mono_16bit_pcm() {
        let w = WavWriter::new(Cursor::new(Vec::new())).unwrap();
        let bytes = w.finalize().unwrap().into_inner();

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(u32_le(&bytes, 16), 16, "fmt chunk size");
        assert_eq!(u16_le(&bytes, 20), 1, "PCM format tag");
        assert_eq!(u16_le(&bytes, 22), 1, "mono");
        assert_eq!(u32_le(&bytes, 24), 16_000, "sample rate");
        assert_eq!(u32_le(&bytes, 28), 32_000, "byte rate = 16000*1*2");
        assert_eq!(u16_le(&bytes, 32), 2, "block align = 1ch*16bit");
        assert_eq!(u16_le(&bytes, 34), 16, "bits per sample");
        assert_eq!(&bytes[36..40], b"data");
    }

    #[test]
    fn start_writes_placeholder_sizes() {
        // Before finalize, both size fields carry the sentinel so a forced-kill
        // partial is detectable on next launch.
        let w = WavWriter::new(Cursor::new(Vec::new())).unwrap();
        let bytes = w.get_ref().get_ref();
        assert_eq!(u32_le(bytes, 4), PLACEHOLDER_SIZE, "RIFF size sentinel");
        assert_eq!(u32_le(bytes, 40), PLACEHOLDER_SIZE, "data size sentinel");
    }

    #[test]
    fn finalize_patches_sizes_for_known_pcm() {
        let mut w = WavWriter::new(Cursor::new(Vec::new())).unwrap();
        let samples = [0i16; 100];
        w.append(&samples).unwrap();
        let bytes = w.finalize().unwrap().into_inner();

        let data_bytes = 100 * 2;
        assert_eq!(u32_le(&bytes, 40), data_bytes, "data chunk size");
        assert_eq!(u32_le(&bytes, 4), 36 + data_bytes, "RIFF size = 36 + data");
        assert_eq!(bytes.len() as u32, 44 + data_bytes, "header + payload");
    }

    #[test]
    fn repair_recovers_unfinalized_file() {
        // Simulate a forced kill: header + PCM on disk, never finalized.
        let mut w = WavWriter::new(Cursor::new(Vec::new())).unwrap();
        w.append(&[0i16; 50]).unwrap();
        let partial = w.get_ref().get_ref().clone();

        let mut stream = Cursor::new(partial);
        let repaired = repair(&mut stream).unwrap();
        assert!(repaired, "sentinel-bearing file should be repaired");

        let bytes = stream.into_inner();
        let data_bytes = 50 * 2;
        assert_eq!(u32_le(&bytes, 40), data_bytes, "data size recomputed");
        assert_eq!(u32_le(&bytes, 4), 36 + data_bytes, "RIFF size recomputed");
    }

    #[test]
    fn repair_rounds_down_to_whole_frame() {
        // 5 trailing PCM bytes = 2 whole 2-byte frames + 1 stray byte; the size
        // field must count only whole frames (4 bytes).
        let mut bytes = vec![0u8; 44 + 5];
        bytes[0..4].copy_from_slice(b"RIFF");
        bytes[8..12].copy_from_slice(b"WAVE");
        bytes[12..16].copy_from_slice(b"fmt ");
        bytes[36..40].copy_from_slice(b"data");
        bytes[4..8].copy_from_slice(&PLACEHOLDER_SIZE.to_le_bytes());
        bytes[40..44].copy_from_slice(&PLACEHOLDER_SIZE.to_le_bytes());

        let mut stream = Cursor::new(bytes);
        assert!(repair(&mut stream).unwrap());
        let out = stream.into_inner();
        assert_eq!(u32_le(&out, 40), 4, "rounded down to 2 whole frames");
        assert_eq!(u32_le(&out, 4), 36 + 4);
    }

    #[test]
    fn repair_is_noop_on_finalized_file() {
        let mut w = WavWriter::new(Cursor::new(Vec::new())).unwrap();
        w.append(&[0i16; 10]).unwrap();
        let finalized = w.finalize().unwrap().into_inner();

        let mut stream = Cursor::new(finalized.clone());
        assert!(!repair(&mut stream).unwrap(), "already finalized");
        assert_eq!(stream.into_inner(), finalized, "bytes unchanged");
    }
}
