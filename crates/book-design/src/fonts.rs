// SPDX-License-Identifier: MIT OR Apache-2.0
//! The fonts Book ships, by role. Fonts are named only in `ATTRIBUTION.md` and their own files
//! under `assets/fonts/`; everywhere else, including here, they go by role (plan section 4,
//! rule 5). Each role is a folder of `regular`, `italic`, `bold` and `bold-italic` faces with its
//! license beside them; the family name is read from the font files when they are used.

/// A font role a design can name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// A classic old-style serif with small caps, old-style figures and fleurons.
    ClassicSerif,
}

impl Role {
    /// Every role, in order.
    pub const ALL: [Role; 1] = [Role::ClassicSerif];

    /// The role's key in design files and its folder under `assets/fonts/`.
    pub fn key(self) -> &'static str {
        match self {
            Role::ClassicSerif => "classic-serif",
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
        }
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
    ]
}

/// The bundled faces of one role.
pub fn of_role(role: Role) -> Vec<Face> {
    bundled().into_iter().filter(|f| f.role == role).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_has_its_four_faces_and_keys_round_trip() {
        for role in Role::ALL {
            assert_eq!(of_role(role).len(), 4, "{role:?}");
            assert_eq!(Role::from_key(role.key()), Some(role));
            for f in of_role(role) {
                assert!(f.bytes.len() > 1000 && (f.bytes.starts_with(&[0, 1, 0, 0]) || f.bytes.starts_with(b"OTTO")));
            }
        }
    }
}
