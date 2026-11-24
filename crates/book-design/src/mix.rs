// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mixing designs (DSN-3) and the user's own designs (DSN-4): a design's body, chapter opening,
//! scene breaks or running heads taken from another; the result saved under the user's name in
//! the project, and exported and imported as one file.

use std::collections::BTreeMap;

use wordsmith_book_model::{DesignChoice, DesignPart};

use crate::design::{self, Design, Error};

/// `base` with each part in `parts` taken from its design.
pub fn mix(base: &Design, parts: &[(DesignPart, &Design)]) -> Design {
    let mut out = base.clone();
    for (part, from) in parts {
        match part {
            DesignPart::Body => {
                out.body = from.body.clone();
                out.ebook.justify = from.ebook.justify;
                out.ebook.indent = from.ebook.indent;
            }
            DesignPart::ChapterOpening => {
                out.chapter = from.chapter.clone();
                out.part = from.part.clone();
                out.ebook.first_line = from.ebook.first_line;
                out.ebook.heading_font = from.ebook.heading_font;
            }
            DesignPart::SceneBreaks => out.scene_break = from.scene_break.clone(),
            DesignPart::RunningHeads => out.heads = from.heads.clone(),
        }
    }
    out
}

/// The design `id`: a project's own (`own`, by id, as file text) or a built-in one.
pub fn find(id: &str, own: &BTreeMap<String, String>) -> Result<Design, Error> {
    if let Some(text) = own.get(id) {
        return design::read(text, &crate::known_role);
    }
    crate::builtin::get(id).ok_or_else(|| Error::NotADesign(format!("there is no design “{id}”")))
}

/// The design a book's choice makes: its design with the mixed-in parts.
pub fn resolve(choice: &DesignChoice, own: &BTreeMap<String, String>) -> Result<Design, Error> {
    let base = find(&choice.id, own)?;
    let mut parts = Vec::new();
    for (part, from) in &choice.mix {
        parts.push((*part, find(from, own)?));
    }
    let refs: Vec<(DesignPart, &Design)> = parts.iter().map(|(p, d)| (*p, d)).collect();
    let mut d = mix(&base, &refs);
    if !choice.mix.is_empty() {
        d.name = format!("{} (mixed)", base.name);
    }
    Ok(d)
}

/// A design id not yet taken in `own` or by a built-in design, from `wanted`.
pub fn free_id(wanted: &str, own: &BTreeMap<String, String>) -> String {
    let base: String = wanted.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect::<String>().trim_matches('-').to_owned();
    let base = if base.is_empty() { "my-design".to_owned() } else { base };
    let taken = |id: &str| own.contains_key(id) || crate::builtin::FILES.iter().any(|(b, _)| *b == id);
    if !taken(&base) {
        return base;
    }
    (2..).map(|n| format!("{base}-{n}")).find(|id| !taken(id)).unwrap_or(base)
}
