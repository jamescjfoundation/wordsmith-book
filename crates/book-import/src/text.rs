// SPDX-License-Identifier: MIT OR Apache-2.0
//! Plain text (IMP-3): paragraphs at blank lines, a single line break inside a paragraph read as a
//! space unless the paragraph is verse (short, indented lines), `_underscores_` read as italic
//! (rule `underscores-italic`, as public-domain e-texts mark it), and two or more blank lines read
//! as a page break, so a title after a wide gap can start a chapter. Structure detection then
//! finds chapters by their lines.

use std::sync::Arc;

use jcj_doc::props::CharProps;
use jcj_doc::{Block as DocBlock, Document, Paragraph};
use wordsmith_book_model::ImportRules;

/// The rule that reads `_underscores_` as italic.
pub const UNDERSCORES: &str = "underscores-italic";

fn is_verse(lines: &[&str]) -> bool {
    lines.len() >= 2 && lines.iter().all(|l| l.trim().chars().count() <= 60) && lines.iter().any(|l| l.starts_with("  "))
}

fn paragraph(text: &str, italics: bool) -> Paragraph {
    let mut p = Paragraph::new();
    if !italics || !text.contains('_') {
        let _ = p.insert_text(0, text, &CharProps::default());
        return p;
    }
    let italic = CharProps { italic: Some(true), ..CharProps::default() };
    let roman = CharProps::default();
    for (i, piece) in text.split('_').enumerate() {
        if !piece.is_empty() {
            let at = p.text.len();
            let _ = p.insert_text(at, piece, if i % 2 == 1 { &italic } else { &roman });
        }
    }
    p
}

/// Whether the rules read underscores as italic.
pub fn underscores(rules: &ImportRules) -> bool {
    !rules.off.iter().any(|r| r == UNDERSCORES)
}

/// Read plain text into the core's document model.
pub fn read(text: &str, rules: &ImportRules) -> Document {
    let italics = underscores(rules);
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut doc = jcj_doc::new_document();
    let mut body: Vec<Arc<DocBlock>> = Vec::new();
    let mut lines: Vec<&str> = Vec::new();
    let mut blank = 0usize;
    let mut gap_before = 0usize;
    let flush = |lines: &mut Vec<&str>, gap: usize, body: &mut Vec<Arc<DocBlock>>| {
        if lines.is_empty() {
            return;
        }
        let joined = if is_verse(lines) {
            lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join("\n")
        } else {
            lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ")
        };
        let mut p = paragraph(&joined, italics);
        if gap >= 2 {
            p.props.page_break_before = Some(true);
        }
        body.push(Arc::new(DocBlock::Para(p)));
        lines.clear();
    };
    for line in text.lines() {
        if line.trim().is_empty() {
            if !lines.is_empty() {
                flush(&mut lines, gap_before, &mut body);
            }
            blank += 1;
            continue;
        }
        if lines.is_empty() {
            gap_before = blank;
        }
        blank = 0;
        lines.push(line);
    }
    flush(&mut lines, gap_before, &mut body);
    if body.is_empty() {
        body.push(Arc::new(DocBlock::Para(Paragraph::new())));
    }
    doc.body = body;
    doc
}

/// The words of the text as the author wrote them (underscores that mark italic left out when
/// the rule reads them as italic).
pub fn words(text: &str, rules: &ImportRules) -> Vec<String> {
    let t = if underscores(rules) { text.replace('_', "") } else { text.to_owned() };
    jcj_features::verify::words(&t)
}
