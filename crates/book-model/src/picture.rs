// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pictures: their pixel size, read from the file's header without decoding it.

/// A picture's pixel size, read from its PNG or JPEG header.
pub fn size(b: &[u8]) -> Option<(u32, u32)> {
    if b.len() >= 24 && b.starts_with(b"\x89PNG\r\n\x1a\n") {
        let w = u32::from_be_bytes([b[16], b[17], b[18], b[19]]);
        let h = u32::from_be_bytes([b[20], b[21], b[22], b[23]]);
        return (w > 0 && h > 0).then_some((w, h));
    }
    if b.starts_with(&[0xFF, 0xD8]) {
        let mut i = 2;
        while i + 9 < b.len() {
            if b[i] != 0xFF {
                return None;
            }
            let marker = b[i + 1];
            let len = usize::from(u16::from_be_bytes([b[i + 2], b[i + 3]]));
            if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
                let h = u32::from(u16::from_be_bytes([b[i + 5], b[i + 6]]));
                let w = u32::from(u16::from_be_bytes([b[i + 7], b[i + 8]]));
                return (w > 0 && h > 0).then_some((w, h));
            }
            i += 2 + len;
        }
    }
    None
}
