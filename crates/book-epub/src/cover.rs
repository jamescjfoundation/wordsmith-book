// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ebook cover (EBK-5, SPEC 6.10): the image declared as `cover-image`, a cover page first in
//! the reading order, and checks of its size against the rule.

use wordsmith_book_model::picture;

use crate::package::{Item, escape};

/// The smallest long side, in pixels.
pub const MIN_LONG_SIDE: u32 = 1600;
/// The largest file, in bytes.
pub const MAX_BYTES: usize = 4 << 20;
/// The ratio of height to width the rule asks for, and how far from it is still "about".
pub const RATIO: (f64, f64) = (1.6, 0.1);

/// What is wrong with a cover image, in words for the build report (empty when nothing is).
pub fn problems(bytes: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let Some((w, h)) = picture::size(bytes) else {
        return vec!["the cover is not a PNG or JPEG image Book can read".into()];
    };
    if w.max(h) < MIN_LONG_SIDE {
        out.push(format!("the cover is {w} × {h} pixels; its long side should be at least {MIN_LONG_SIDE}"));
    }
    if bytes.len() > MAX_BYTES {
        out.push(format!("the cover is {:.1} MB; it should be at most 4 MB", bytes.len() as f64 / f64::from(1u32 << 20)));
    }
    let ratio = f64::from(h) / f64::from(w);
    if (ratio - RATIO.0).abs() > RATIO.1 {
        out.push(format!("the cover's height is {ratio:.2} times its width; about {:.1} is expected", RATIO.0));
    }
    out
}

/// The cover image's item and its page.
pub fn items(bytes: Vec<u8>, media_type: &str, title: &str, lang: &str) -> (Item, Item) {
    let ext = if media_type == "image/png" { "png" } else { "jpg" };
    let href = format!("images/cover.{ext}");
    let (w, h) = picture::size(&bytes).unwrap_or((1600, 2560));
    let lang = escape(lang);
    let page = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n<head>\n<meta charset=\"UTF-8\"/>\n<meta name=\"viewport\" content=\"width={w}, height={h}\"/>\n<title>Cover</title>\n<style>body {{ margin: 0; text-align: center; }} img {{ max-width: 100%; max-height: 100vh; }}</style>\n</head>\n<body epub:type=\"cover\">\n<section id=\"cover\" epub:type=\"cover\"><img src=\"../{href}\" alt=\"Cover of {}\"/></section>\n</body>\n</html>\n",
        escape(title)
    );
    (
        Item { id: "cover-image".into(), href, media_type: media_type.into(), properties: vec!["cover-image".into()], bytes },
        Item {
            id: "cover".into(),
            href: "text/cover.xhtml".into(),
            media_type: "application/xhtml+xml".into(),
            properties: Vec::new(),
            bytes: page.into_bytes(),
        },
    )
}
