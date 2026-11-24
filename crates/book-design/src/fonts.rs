// SPDX-License-Identifier: MIT OR Apache-2.0
//! The fonts Book ships, by role. Fonts are named only in `ATTRIBUTION.md` and their own files
//! under `assets/fonts/`; everywhere else, including here, they go by role (plan section 4,
//! rule 5). Each role is a folder of faces (`regular`, `italic`, `bold`, `bold-italic`) with its
//! license beside them; the family name is read from the font files when they are used.
//!
//! [`has_char`] and [`embedding`] read a font file's character map and embedding bits, bounded
//! and without trusting the file, for design checks and preflight.

/// A font role a design can name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// A classic old-style serif with small caps, old-style figures and fleurons.
    ClassicSerif,
    /// A sturdy serif made for long reading, with small caps.
    ReadingSerif,
    /// A high-contrast display serif whose lower case is small capitals, for titles.
    Display,
    /// A small-capital display serif, for titles.
    Inscription,
    /// A sans serif with small caps, for headings.
    Sans,
}

impl Role {
    /// Every role, in order.
    pub const ALL: [Role; 5] = [Role::ClassicSerif, Role::ReadingSerif, Role::Display, Role::Inscription, Role::Sans];

    /// The role's key in design files and its folder under `assets/fonts/`.
    pub fn key(self) -> &'static str {
        match self {
            Role::ClassicSerif => "classic-serif",
            Role::ReadingSerif => "reading-serif",
            Role::Display => "display",
            Role::Inscription => "inscription",
            Role::Sans => "sans",
        }
    }

    /// The role for a key.
    pub fn from_key(key: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.key() == key)
    }

    /// The role's name as the interface shows it.
    pub fn title(self) -> &'static str {
        match self {
            Role::ClassicSerif => "Classic serif",
            Role::ReadingSerif => "Reading serif",
            Role::Display => "Display",
            Role::Inscription => "Display capitals",
            Role::Sans => "Sans serif",
        }
    }

    /// Whether the role is for headings only (no italic, perhaps no lower case).
    pub fn headings_only(self) -> bool {
        matches!(self, Role::Display | Role::Inscription)
    }
}

/// One bundled font file.
#[derive(Clone, Copy, Debug)]
pub struct Face {
    /// Its role.
    pub role: Role,
    /// `regular`, `italic`, `bold` or `bold-italic`.
    pub style: &'static str,
    /// The font file's bytes.
    pub bytes: &'static [u8],
}

macro_rules! face {
    ($role:expr, $dir:literal, $style:literal) => {
        Face { role: $role, style: $style, bytes: include_bytes!(concat!("../../../assets/fonts/", $dir, "/", $style, ".ttf")) }
    };
}

/// Every bundled font file.
pub fn bundled() -> Vec<Face> {
    vec![
        face!(Role::ClassicSerif, "classic-serif", "regular"),
        face!(Role::ClassicSerif, "classic-serif", "italic"),
        face!(Role::ClassicSerif, "classic-serif", "bold"),
        face!(Role::ClassicSerif, "classic-serif", "bold-italic"),
        face!(Role::ReadingSerif, "reading-serif", "regular"),
        face!(Role::ReadingSerif, "reading-serif", "italic"),
        face!(Role::ReadingSerif, "reading-serif", "bold"),
        face!(Role::ReadingSerif, "reading-serif", "bold-italic"),
        face!(Role::Display, "display", "regular"),
        face!(Role::Display, "display", "italic"),
        face!(Role::Display, "display", "bold"),
        face!(Role::Display, "display", "bold-italic"),
        face!(Role::Inscription, "inscription", "regular"),
        face!(Role::Inscription, "inscription", "bold"),
        face!(Role::Sans, "sans", "regular"),
        face!(Role::Sans, "sans", "italic"),
        face!(Role::Sans, "sans", "bold"),
        face!(Role::Sans, "sans", "bold-italic"),
    ]
}

/// The bundled faces of one role.
pub fn of_role(role: Role) -> Vec<Face> {
    bundled().into_iter().filter(|f| f.role == role).collect()
}

/// The regular face of a role.
pub fn regular(role: Role) -> Option<Face> {
    bundled().into_iter().find(|f| f.role == role && f.style == "regular")
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*b.get(at)?, *b.get(at + 1)?]))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes([*b.get(at)?, *b.get(at + 1)?, *b.get(at + 2)?, *b.get(at + 3)?]))
}

/// A table of a font file: its offset and length.
fn table(b: &[u8], tag: &[u8; 4]) -> Option<(usize, usize)> {
    let n = usize::from(u16_at(b, 4)?).min(512);
    for i in 0..n {
        let rec = 12 + 16 * i;
        if b.get(rec..rec + 4)? == tag {
            let off = usize::try_from(u32_at(b, rec + 8)?).ok()?;
            let len = usize::try_from(u32_at(b, rec + 12)?).ok()?;
            return (off.checked_add(len)? <= b.len()).then_some((off, len));
        }
    }
    None
}

/// Whether a font file maps character `c` to a glyph (its `cmap` table, formats 4 and 12).
pub fn has_char(b: &[u8], c: char) -> bool {
    let cp = u32::from(c);
    let Some((cmap, _)) = table(b, b"cmap") else { return false };
    let n = u16_at(b, cmap + 2).map_or(0, usize::from).min(64);
    for i in 0..n {
        let Some(off) = u32_at(b, cmap + 4 + 8 * i + 4).and_then(|o| usize::try_from(o).ok()) else { continue };
        let st = cmap + off;
        match u16_at(b, st) {
            Some(4) if cp <= 0xFFFF => {
                let segs = u16_at(b, st + 6).map_or(0, usize::from) / 2;
                for s in 0..segs.min(8192) {
                    let end = u16_at(b, st + 14 + 2 * s).map_or(0, u32::from);
                    let start = u16_at(b, st + 16 + 2 * segs + 2 * s).map_or(1, u32::from);
                    if start <= cp && cp <= end && end != 0xFFFF {
                        return true;
                    }
                }
            }
            Some(12) => {
                let groups = u32_at(b, st + 12).and_then(|g| usize::try_from(g).ok()).unwrap_or(0).min(1 << 16);
                for g in 0..groups {
                    let at = st + 16 + 12 * g;
                    let (Some(a), Some(z)) = (u32_at(b, at), u32_at(b, at + 4)) else { break };
                    if a <= cp && cp <= z {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// The font's family name (its `name` table: the typographic family, else the family).
pub fn family_name(b: &[u8]) -> Option<String> {
    let (name, len) = table(b, b"name")?;
    let count = usize::from(u16_at(b, name + 2)?).min(1024);
    let strings = name + usize::from(u16_at(b, name + 4)?);
    let mut best: Option<(u16, String)> = None;
    for i in 0..count {
        let rec = name + 6 + 12 * i;
        let (platform, encoding, id) = (u16_at(b, rec)?, u16_at(b, rec + 2)?, u16_at(b, rec + 6)?);
        if id != 1 && id != 16 {
            continue;
        }
        let (l, off) = (usize::from(u16_at(b, rec + 8)?), usize::from(u16_at(b, rec + 10)?));
        let start = strings + off;
        if start + l > name + len {
            continue;
        }
        let raw = b.get(start..start + l)?;
        let text = if platform == 3 || (platform == 0 && encoding <= 4) {
            let units: Vec<u16> = raw.as_chunks::<2>().0.iter().map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
            String::from_utf16(&units).ok()?
        } else if platform == 1 {
            raw.iter().map(|&c| char::from(c)).collect()
        } else {
            continue;
        };
        // The typographic family (16) wins over the family (1).
        if best.as_ref().is_none_or(|(bid, _)| id > *bid) {
            best = Some((id, text));
        }
    }
    best.map(|(_, t)| t)
}

/// What a font's embedding bits (`OS/2` `fsType`) allow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Embedding {
    /// Installable: may be embedded freely.
    Installable,
    /// Restricted: may not be embedded.
    Restricted,
    /// Preview and print, or editable: may be embedded in a document.
    Allowed,
    /// No `OS/2` table: unknown.
    Unknown,
}

/// The font's embedding permission.
pub fn embedding(b: &[u8]) -> Embedding {
    let Some((os2, len)) = table(b, b"OS/2") else { return Embedding::Unknown };
    if len < 10 {
        return Embedding::Unknown;
    }
    match u16_at(b, os2 + 8).map(|f| f & 0x000F) {
        Some(0) => Embedding::Installable,
        Some(2) => Embedding::Restricted,
        Some(_) => Embedding::Allowed,
        None => Embedding::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_has_a_regular_face_and_keys_round_trip() {
        for role in Role::ALL {
            let faces = of_role(role);
            assert!(faces.iter().any(|f| f.style == "regular"), "{role:?}");
            assert!(faces.len() == 4 || role.headings_only(), "{role:?}");
            assert_eq!(Role::from_key(role.key()), Some(role));
            for f in faces {
                assert!(f.bytes.len() > 1000 && (f.bytes.starts_with(&[0, 1, 0, 0]) || f.bytes.starts_with(b"OTTO")));
                assert!(has_char(f.bytes, 'A'));
                assert!(!has_char(f.bytes, '\u{E000}'));
            }
        }
        assert!(!has_char(b"not a font", 'A'));
        assert_eq!(embedding(b"tiny"), Embedding::Unknown);
    }
}
