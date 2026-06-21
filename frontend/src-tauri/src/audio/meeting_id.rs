//! Time-ordered identifier for `<meeting-id>.wav` filenames.

use uuid::Uuid;

/// Generate a time-ordered meeting id for the WAV filename.
///
/// UUID v7 embeds a millisecond Unix timestamp in its high bits, so the
/// hyphenated string form sorts lexicographically in creation order — the
/// audio directory lists newest-last without any extra bookkeeping. The
/// hyphenated form contains only `[0-9a-f-]`, so it is a safe NTFS filename
/// component.
pub fn generate() -> String {
    Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_id_is_filename_safe() {
        let id = generate();
        assert!(!id.is_empty());
        // Must be usable as a Windows filename component: no path separators
        // or characters illegal on NTFS.
        for bad in ['/', '\\', ':', '*', '?', '"', '<', '>', '|'] {
            assert!(!id.contains(bad), "id {id:?} contains illegal char {bad:?}");
        }
    }

    #[test]
    fn successive_ids_are_unique() {
        assert_ne!(generate(), generate());
    }

    #[test]
    fn ids_sort_in_creation_order() {
        // UUID v7 is time-ordered: an id minted later must sort lexicographically
        // at or after one minted earlier, so the audio directory lists by time.
        let earlier = generate();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let later = generate();
        assert!(later > earlier, "{later:?} should sort after {earlier:?}");
    }
}
