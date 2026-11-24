// SPDX-License-Identifier: MIT OR Apache-2.0
//! Accessibility (EBK-3, SPEC 6.5 and 6.6): the package's accessibility metadata, made from what
//! the ebook holds, and the alternative-text rule that blocks export.

use wordsmith_book_format::assemble::Assembled;
use wordsmith_manuscript_model::Block;

use crate::in_ebook;

/// The pictures in the ebook that have no alternative text and are not marked decorative, by
/// their path in the source, once each, in reading order.
pub fn missing_alt_text(a: &Assembled, decorative: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in a.pieces().filter(|p| in_ebook(p)) {
        for b in &p.blocks {
            if let Block::Image { alt, src, .. } = b
                && alt.trim().is_empty()
                && !decorative.contains(src)
                && !out.contains(src)
            {
                out.push(src.clone());
            }
        }
    }
    out
}

/// What the accessibility metadata describes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Content {
    /// The ebook shows pictures that carry meaning (not decorative).
    pub pictures: bool,
    /// It has a page list (from 0.2).
    pub page_list: bool,
}

/// The package's accessibility `meta` elements, as (property, value), in order.
pub fn metadata(content: Content, certifier: &str) -> Vec<(String, String)> {
    let mut m: Vec<(&str, String)> = vec![("schema:accessMode", "textual".into())];
    if content.pictures {
        m.push(("schema:accessMode", "visual".into()));
    }
    // Every picture has alternative text, so text alone is enough.
    m.push(("schema:accessModeSufficient", "textual".into()));
    for f in ["structuralNavigation", "tableOfContents", "readingOrder"] {
        m.push(("schema:accessibilityFeature", f.into()));
    }
    if content.pictures {
        m.push(("schema:accessibilityFeature", "alternativeText".into()));
    }
    if content.page_list {
        m.push(("schema:accessibilityFeature", "pageNavigation".into()));
    }
    m.push(("schema:accessibilityHazard", "none".into()));
    let pictures = if content.pictures { " Every picture has alternative text." } else { "" };
    m.push((
        "schema:accessibilitySummary",
        format!(
            "This ebook has a table of contents, landmarks and headings for its structure, and its reading order follows the book.{pictures} Text can be resized and restyled by the reading system."
        ),
    ));
    m.push(("dcterms:conformsTo", "EPUB Accessibility 1.1 - WCAG 2.2 Level AA".into()));
    if !certifier.is_empty() {
        m.push(("a11y:certifiedBy", certifier.to_owned()));
    }
    m.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
}
