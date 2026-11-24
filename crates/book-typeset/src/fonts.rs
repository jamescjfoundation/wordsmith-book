// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's bundled fonts, loaded into the core's layout once per run, by role. System fonts are
//! never used for print, so output does not depend on the machine.

use std::sync::OnceLock;

use wordsmith_book_design::fonts::{Role, bundled};

static FAMILIES: OnceLock<Vec<(Role, String)>> = OnceLock::new();

fn load() -> Vec<(Role, String)> {
    jcj_layout::fonts::set_system_fallback(false);
    let mut out: Vec<(Role, String)> = Vec::new();
    for face in bundled() {
        let name = format!("{}-{}.ttf", face.role.key(), face.style);
        let mut families = jcj_layout::fonts::install_user_font(&name, face.bytes).unwrap_or_default();
        // A family the core's layout already has (it bundles some open fonts) is not added again:
        // its name comes from the file, and the loaded faces are used.
        if families.is_empty()
            && let Some(f) = wordsmith_book_design::fonts::family_name(face.bytes)
            && jcj_layout::fonts::is_loaded(&f)
        {
            families.push(f);
        }
        for family in families {
            if !out.iter().any(|(r, f)| *r == face.role && *f == family) {
                out.push((face.role, family));
            }
        }
    }
    out
}

/// The family name of the bundled font of `role` (loading the bundled fonts the first time).
pub fn family(role: Role) -> Option<String> {
    FAMILIES.get_or_init(load).iter().find(|(r, _)| *r == role).map(|(_, f)| f.clone())
}
