// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's conversion between the book view's semantic text (Manuscript's blocks, marks and
//! notes) and the core's document model (`jcj_doc::Document`), with Book's data under `book.`
//! extension keys (core decision 0006). Book needs it to import files (M2.2), fix text in place
//! (M3.4) and draw the ebook preview (M7.2). Manuscript has a conversion of its own, built
//! against its own core version; this one is Book's, never a copy.
//!
//! Text → document → text is exact for every block and mark the book view holds. A document
//! with formatting the book view cannot hold (fonts, sizes, colors, underline, alignment, ...)
//! converts with that formatting dropped and listed ([`Dropped`]).

use std::collections::BTreeMap;
use std::sync::Arc;

use jcj_doc::para::{InlineObject, NoteKind, OBJ};
use jcj_doc::props::CharProps;
use jcj_doc::{Block as DocBlock, Document, ExtKey, Paragraph, PartKind};
use wordsmith_manuscript_model::{Block, Fence, ImageRole, Inline, Inlines, Marks, Note, Text};

/// What kind of block a paragraph belongs to.
pub const BLOCK: ExtKey<String> = ExtKey::new("book.block");
/// Which block a paragraph belongs to, for blocks of several paragraphs (quotes, verse, asides).
pub const GROUP: ExtKey<u64> = ExtKey::new("book.group");
/// An aside's kind.
pub const ASIDE: ExtKey<String> = ExtKey::new("book.aside");
/// A heading's level.
pub const LEVEL: ExtKey<u8> = ExtKey::new("book.level");
/// An image's path, alternative text and role.
pub const IMAGE: ExtKey<(String, String, String)> = ExtKey::new("book.image");
/// A raw block's format and content.
pub const RAW: ExtKey<(String, String)> = ExtKey::new("book.raw");
/// A comment anchored on a run.
pub const COMMENT: ExtKey<String> = ExtKey::new("book.comment");
/// A revision pass a run was written in.
pub const REV: ExtKey<u32> = ExtKey::new("book.rev");
/// Notes no reference points at, kept as they are.
pub const UNREFERENCED: ExtKey<Vec<Note>> = ExtKey::new("book.unreferenced");

/// Formatting a document had that the book view cannot hold, by kind, with how often.
pub type Dropped = BTreeMap<&'static str, usize>;

fn chars_of(m: &Marks) -> CharProps {
    let mut c = CharProps::default();
    if m.italic {
        c.italic = Some(true);
    }
    if m.bold {
        c.bold = Some(true);
    }
    if m.small_caps {
        c.small_caps = Some(true);
    }
    if m.strike {
        c.strike = Some(true);
    }
    c.lang.clone_from(&m.lang);
    c.link.clone_from(&m.link);
    if let Some(id) = &m.comment {
        let _ = COMMENT.set(&mut c, id);
    }
    if let Some(r) = m.rev {
        let _ = REV.set(&mut c, &r);
    }
    c
}

fn marks_of(c: &CharProps, dropped: &mut Dropped) -> Marks {
    let mut note = |what: &'static str, present: bool| {
        if present {
            *dropped.entry(what).or_default() += 1;
        }
    };
    note("font", c.font.is_some());
    note("size", c.size.is_some());
    note("color", c.color.is_some() || c.highlight.is_some() || c.shading.is_some());
    note("underline", c.underline.is_some());
    note("capitals", c.caps == Some(true));
    note("raised or lowered text", c.vert_align.is_some() || c.position.is_some());
    note("spacing", c.spacing.is_some() || c.scale.is_some() || c.kern.is_some());
    note("hidden text", c.hidden == Some(true));
    Marks {
        italic: c.italic == Some(true),
        bold: c.bold == Some(true),
        small_caps: c.small_caps == Some(true),
        strike: c.strike == Some(true) || c.double_strike == Some(true),
        lang: c.lang.clone().filter(|l| !l.is_empty()),
        link: c.link.clone().filter(|l| !l.is_empty()),
        comment: COMMENT.get_or_none(c),
        rev: REV.get_or_none(c),
    }
}

struct Out<'a> {
    doc: &'a mut Document,
    group: u64,
}

impl Out<'_> {
    fn para(&mut self, content: &[Inline], notes: &[Note], kind: &str, group: Option<u64>) -> Paragraph {
        let mut p = Paragraph::new();
        for i in content {
            match i {
                Inline::Span { text, marks } => {
                    let at = p.text.len();
                    let _ = p.insert_text(at, text, &chars_of(marks));
                }
                Inline::NoteRef { label, marks } => {
                    let body: Vec<Arc<DocBlock>> = notes
                        .iter()
                        .find(|n| &n.label == label)
                        .map(|n| n.paras.iter().map(|np| Arc::new(DocBlock::Para(self.para(np, &[], "note", None)))).collect())
                        .unwrap_or_default();
                    let id = self.doc.add_part(PartKind::Footnote, body);
                    let at = p.text.len();
                    let _ = p.insert_object(at, InlineObject::NoteRef { kind: NoteKind::Footnote, id, custom: label.clone() }, &chars_of(marks));
                }
            }
        }
        let _ = BLOCK.set(&mut p, &kind.to_owned());
        if let Some(g) = group {
            let _ = GROUP.set(&mut p, &g);
        }
        p
    }

    fn next_group(&mut self) -> u64 {
        self.group += 1;
        self.group
    }
}

/// The text as a document: one paragraph per paragraph of the text, notes as footnotes.
pub fn to_document(text: &Text) -> Document {
    let mut doc = jcj_doc::new_document();
    let mut body: Vec<Paragraph> = Vec::new();
    {
        let mut out = Out { doc: &mut doc, group: 0 };
        for b in &text.blocks {
            match b {
                Block::Para(c) => body.push(out.para(c, &text.notes, "para", None)),
                Block::Heading { level, content } => {
                    let mut p = out.para(content, &text.notes, "heading", None).styled(&format!("Heading{}", (*level).clamp(1, 3)));
                    let _ = LEVEL.set(&mut p, level);
                    body.push(p);
                }
                Block::SceneBreak => body.push(out.para(&[], &[], "break", None)),
                Block::Quote(ps) => {
                    let g = out.next_group();
                    body.extend(ps.iter().map(|c| out.para(c, &text.notes, "quote", Some(g))));
                }
                Block::Fenced { kind, paras } => {
                    let g = out.next_group();
                    for c in paras {
                        let mut p = out.para(c, &text.notes, if *kind == Fence::Verse { "verse" } else { "aside" }, Some(g));
                        if let Fence::Aside(Some(k)) = kind {
                            let _ = ASIDE.set(&mut p, k);
                        }
                        body.push(p);
                    }
                }
                Block::Image { alt, src, role } => {
                    let mut p = out.para(&[], &[], "image", None);
                    let _ = IMAGE.set(&mut p, &(src.clone(), alt.clone(), role.key().to_owned()));
                    body.push(p);
                }
                Block::Raw { format, text } => {
                    let mut p = out.para(&[], &[], "raw", None);
                    let _ = RAW.set(&mut p, &(format.clone(), text.clone()));
                    body.push(p);
                }
            }
        }
    }
    let referenced: Vec<&str> = text
        .inline_runs()
        .iter()
        .flat_map(|r| r.iter())
        .filter_map(|i| if let Inline::NoteRef { label, .. } = i { Some(label.as_str()) } else { None })
        .collect();
    let unreferenced: Vec<Note> = text.notes.iter().filter(|n| !referenced.contains(&n.label.as_str())).cloned().collect();
    if !unreferenced.is_empty() {
        let _ = UNREFERENCED.set(&mut doc, &unreferenced);
    }
    if body.is_empty() {
        body.push(Paragraph::new());
        let _ = BLOCK.set(&mut body[0], &"empty".to_owned());
    }
    doc.body = body.into_iter().map(|p| Arc::new(DocBlock::Para(p))).collect();
    doc
}

fn inlines_of(doc: &Document, p: &Paragraph, notes: &mut Vec<Note>, dropped: &mut Dropped) -> Inlines {
    let objects: BTreeMap<usize, &InlineObject> = p.objects_with_offsets().collect();
    let mut out: Inlines = Vec::new();
    let mut at = 0usize;
    for run in &p.runs {
        let end = (at + run.len).min(p.text.len());
        let piece = p.text.get(at..end).unwrap_or("");
        let marks = marks_of(&run.props, dropped);
        let mut buf = String::new();
        for (i, ch) in piece.char_indices() {
            if ch == OBJ {
                if !buf.is_empty() {
                    out.push(Inline::marked(std::mem::take(&mut buf), marks.clone()));
                }
                match objects.get(&(at + i)) {
                    Some(InlineObject::NoteRef { id, custom, .. }) => {
                        let label = if custom.is_empty() { (notes.len() + 1).to_string() } else { custom.clone() };
                        if !notes.iter().any(|n| n.label == label) {
                            let paras = doc
                                .parts
                                .get(id)
                                .map(|part| {
                                    part.blocks.iter().filter_map(|b| b.as_para()).map(|np| inlines_of(doc, np, &mut Vec::new(), dropped)).collect()
                                })
                                .unwrap_or_default();
                            notes.push(Note { label: label.clone(), paras });
                        }
                        out.push(Inline::NoteRef { label, marks: marks.clone() });
                    }
                    Some(InlineObject::Image { .. }) => *dropped.entry("inline pictures").or_default() += 1,
                    Some(_) | None => *dropped.entry("objects").or_default() += 1,
                }
            } else {
                buf.push(ch);
            }
        }
        if !buf.is_empty() {
            out.push(Inline::marked(buf, marks));
        }
        at = end;
    }
    wordsmith_manuscript_model::text::normalize_inlines(&mut out);
    out
}

fn para_dropped(p: &Paragraph, dropped: &mut Dropped) {
    let props = &p.props;
    let mut note = |what: &'static str, present: bool| {
        if present {
            *dropped.entry(what).or_default() += 1;
        }
    };
    note("alignment", props.align.is_some());
    note("indents", props.indent_left.is_some() || props.indent_right.is_some() || props.indent_first.is_some());
    note("paragraph spacing", props.space_before.is_some() || props.space_after.is_some() || props.line_spacing.is_some());
    note("page breaks", props.page_break_before == Some(true));
    note("borders and shading", props.borders.is_some() || props.shading.is_some());
    note("lists", props.numbering.is_some());
}

/// The document as text, with the formatting that was dropped.
pub fn from_document(doc: &Document) -> (Text, Dropped) {
    let mut dropped = Dropped::new();
    let mut notes: Vec<Note> = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    // The group being built: (kind, group id, aside kind, paragraphs).
    let mut open: Option<(String, u64, Option<String>, Vec<Inlines>)> = None;
    let flush = |open: &mut Option<(String, u64, Option<String>, Vec<Inlines>)>, blocks: &mut Vec<Block>| {
        if let Some((kind, _, aside, paras)) = open.take() {
            blocks.push(match kind.as_str() {
                "quote" => Block::Quote(paras),
                "verse" => Block::Fenced { kind: Fence::Verse, paras },
                _ => Block::Fenced { kind: Fence::Aside(aside), paras },
            });
        }
    };
    for b in &doc.body {
        let Some(p) = b.as_para() else {
            *dropped.entry("tables").or_default() += 1;
            continue;
        };
        let kind = BLOCK.get_or_none(p);
        if kind.is_none() {
            para_dropped(p, &mut dropped);
        }
        let style_level = p.props.style.as_deref().and_then(|s| s.strip_prefix("Heading")).and_then(|n| n.parse::<u8>().ok());
        let kind = kind.unwrap_or_else(|| if style_level.is_some() { "heading".into() } else { "para".into() });
        let content = inlines_of(doc, p, &mut notes, &mut dropped);
        match kind.as_str() {
            "quote" | "verse" | "aside" => {
                let g = GROUP.get_or_none(p).unwrap_or(0);
                let aside = ASIDE.get_or_none(p);
                let same = open.as_ref().is_some_and(|(k, og, _, _)| *k == kind && *og == g && g != 0);
                if !same {
                    flush(&mut open, &mut blocks);
                    open = Some((kind.clone(), g, aside, Vec::new()));
                }
                if let Some((_, _, _, paras)) = open.as_mut() {
                    paras.push(content);
                }
                continue;
            }
            _ => flush(&mut open, &mut blocks),
        }
        match kind.as_str() {
            "heading" => blocks.push(Block::Heading { level: LEVEL.get_or_none(p).or(style_level).unwrap_or(1).clamp(1, 3), content }),
            "break" => blocks.push(Block::SceneBreak),
            "image" => {
                let (src, alt, role) = IMAGE.get_or_none(p).unwrap_or_default();
                blocks.push(Block::Image { alt, src, role: ImageRole::from_key(&role).unwrap_or_default() });
            }
            "raw" => {
                let (format, text) = RAW.get_or_none(p).unwrap_or_default();
                blocks.push(Block::Raw { format, text });
            }
            "empty" => {}
            _ => blocks.push(Block::Para(content)),
        }
    }
    flush(&mut open, &mut blocks);
    notes.extend(UNREFERENCED.get_or_none(doc).unwrap_or_default());
    let mut t = Text { blocks, notes };
    t.normalize();
    (t, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_document_converts_and_lists_what_it_dropped() {
        let mut doc = jcj_doc::new_document();
        let mut p =
            Paragraph::with_text("Hello", CharProps { font: Some("Any".into()), size: Some(12.0), italic: Some(true), ..CharProps::default() });
        p.props.align = Some(jcj_doc::props::Align::Center);
        doc.body =
            vec![Arc::new(DocBlock::Para(p)), Arc::new(DocBlock::Para(Paragraph::with_text("Title", CharProps::default()).styled("Heading2")))];
        let (t, dropped) = from_document(&doc);
        assert_eq!(t.blocks[0], Block::Para(vec![Inline::marked("Hello", Marks::italic())]));
        assert!(matches!(&t.blocks[1], Block::Heading { level: 2, .. }));
        assert_eq!(dropped.get("font"), Some(&1));
        assert_eq!(dropped.get("size"), Some(&1));
        assert_eq!(dropped.get("alignment"), Some(&1));
    }
}
