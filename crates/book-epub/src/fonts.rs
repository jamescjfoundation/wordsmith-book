// SPDX-License-Identifier: MIT OR Apache-2.0
//! Embedded fonts (EBK-4, SPEC 6.9): when the edition embeds fonts, the design's body font and
//! (when the design's ebook choices use it) its heading font go into the package, not
//! obfuscated, and only when the font's license bits allow embedding.

use wordsmith_book_design::design::Design;
use wordsmith_book_design::fonts::{self, Embedding, Role};

use crate::css::Fonts;
use crate::package::Item;

fn weight_style(style: &str) -> (u16, &'static str) {
    match style {
        "bold" => (700, "normal"),
        "italic" => (400, "italic"),
        "bold-italic" => (700, "italic"),
        _ => (400, "normal"),
    }
}

/// The family name the stylesheet gives a role's embedded faces.
pub fn family(role: Role) -> String {
    format!("book-{}", role.key())
}

/// The font files to embed and the stylesheet rules that use them. Fonts whose license bits
/// forbid embedding are left out (and given back by name in the third value).
pub fn embedded(d: &Design) -> (Vec<Item>, Fonts, Vec<String>) {
    let mut roles = Vec::new();
    if let Some(r) = Role::from_key(&d.body.font) {
        roles.push(r);
    }
    if d.ebook.heading_font
        && let Some(r) = Role::from_key(&d.chapter.font)
        && !roles.contains(&r)
    {
        roles.push(r);
    }
    let mut items = Vec::new();
    let mut css = Fonts::default();
    let mut refused = Vec::new();
    for (i, role) in roles.iter().enumerate() {
        let mut any = false;
        for face in fonts::of_role(*role) {
            if fonts::embedding(face.bytes) == Embedding::Restricted {
                refused.push(format!("{} {}", role.title(), face.style));
                continue;
            }
            any = true;
            let href = format!("fonts/{}-{}.ttf", role.key(), face.style);
            let (weight, style) = weight_style(face.style);
            css.faces += &format!(
                "@font-face {{ font-family: \"{}\"; font-weight: {weight}; font-style: {style}; src: url(\"../{href}\"); }}\n",
                family(*role)
            );
            items.push(Item {
                id: format!("font-{}-{}", role.key(), face.style),
                href,
                media_type: "font/ttf".into(),
                properties: Vec::new(),
                bytes: face.bytes.to_vec(),
            });
        }
        if any {
            let generic = if *role == Role::Sans { "sans-serif" } else { "serif" };
            let list = format!("\"{}\", {generic}", family(*role));
            if i == 0 {
                css.body = Some(list);
            } else {
                css.heading = Some(list);
            }
        }
    }
    if css.heading.is_none() && d.ebook.heading_font {
        css.heading.clone_from(&css.body);
    }
    (items, css, refused)
}
