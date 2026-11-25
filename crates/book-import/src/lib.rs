// SPDX-License-Identifier: MIT OR Apache-2.0
//! Imports (plan section 8): DOCX, ODT, RTF, Markdown, text and EPUB, read through the core's
//! readers, given structure ([`detect`]), and written as an owned source: a Manuscript-format
//! project inside the Book project. Every import gives a report (what was made, formatting kept
//! and dropped, the word-for-word check), and an import whose words do not match is refused.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod cleanup;
pub mod detect;
pub mod epub;
pub mod reimport;
pub mod text;

use std::collections::BTreeMap;
use std::sync::Arc;

use jcj_doc::para::{InlineObject, OBJ};
use jcj_doc::{Block as DocBlock, Document, Paragraph};
use jcj_features::verify::{self, Check};
use wordsmith_book_model::ImportRules;
use wordsmith_book_model::document::{Dropped, from_document};
use wordsmith_manuscript_model::{Area, Item, ItemId, ItemText, Kind as ItemKind, Place, Project, Role, Text};

use detect::{Kind, Mark, PAGE_BREAK, Para};

pub use jcj_interchange::Format;

/// Why an import failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The file could not be read as its format.
    Read(String),
    /// The imported words differ from the file's: nothing is written.
    Words(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Read(m) => write!(f, "the file could not be read: {m}"),
            Error::Words(m) => write!(f, "the import was refused because its words would change: {m}"),
        }
    }
}

impl std::error::Error for Error {}

/// What an import made and checked.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    /// Sections made, by kind ("parts", "chapters", "front matter", "back matter", "scene breaks").
    pub made: BTreeMap<&'static str, usize>,
    /// Formatting kept, by kind ("italic", "notes", ...).
    pub kept: BTreeMap<&'static str, usize>,
    /// Formatting dropped, by kind ("font", "size", "empty paragraphs", "page breaks", ...).
    pub dropped: Dropped,
    /// Changes made on the way that the user can turn off ("underline shown as italic").
    pub changes: BTreeMap<&'static str, usize>,
    /// The words of the text, checked.
    pub words: Option<Check>,
    /// The words of the notes, checked.
    pub notes: Option<Check>,
}

impl Report {
    /// Whether every word matched.
    pub fn passed(&self) -> bool {
        self.words.as_ref().is_some_and(Check::passed) && self.notes.as_ref().is_none_or(Check::passed)
    }
}

/// An import: the owned source's project and its report.
#[derive(Clone, Debug, PartialEq)]
pub struct Imported {
    /// The Manuscript-format project to write as the owned source.
    pub project: Project,
    /// What was made and checked.
    pub report: Report,
    /// Sections whose titles the import made up (text before the first heading).
    pub named: std::collections::BTreeSet<ItemId>,
}

/// Read a file of `format` into the core's document.
pub fn read(format: Format, bytes: &[u8]) -> Result<Document, Error> {
    jcj_interchange::import(format, bytes).map_err(|e| Error::Read(e.to_string()))
}

/// The paragraphs of a document as detection sees them.
pub fn paras_of(doc: &Document) -> Vec<Para> {
    let mut out = Vec::with_capacity(doc.body.len());
    let mut after_break = false;
    for b in &doc.body {
        let Some(p) = b.as_para() else {
            out.push(Para::default());
            continue;
        };
        let level = p
            .props
            .style
            .as_deref()
            .and_then(|s| s.strip_prefix("Heading"))
            .and_then(|n| n.trim().parse::<u8>().ok())
            .or(p.props.outline_level.map(|l| l + 1));
        let text = p.text.replace(OBJ, "");
        let bold = !p.runs.is_empty() && p.runs.iter().all(|r| r.props.bold == Some(true) || r.len == 0);
        let centered = p.props.align == Some(jcj_doc::props::Align::Center);
        let has_break = text.contains(PAGE_BREAK);
        out.push(Para {
            text: text.replace(PAGE_BREAK, ""),
            level,
            bold,
            centered,
            after_page_break: after_break || p.props.page_break_before == Some(true),
        });
        after_break = has_break && text.trim_matches(|c: char| c.is_whitespace() || c == PAGE_BREAK).is_empty();
    }
    out
}

/// The detected structure of a document, as (paragraph index, mark).
pub fn structure(doc: &Document, rules: &ImportRules) -> Vec<Mark> {
    detect::detect(&paras_of(doc), rules)
}

struct Builder {
    project: Project,
    next: u64,
    part: Option<ItemId>,
    report: Report,
    named: std::collections::BTreeSet<ItemId>,
}

impl Builder {
    fn id(&mut self) -> ItemId {
        self.next += 1;
        ItemId::from_counter(self.next)
    }

    fn add(&mut self, kind: Kind, title: &str, text: Option<Text>) -> ItemId {
        let id = self.id();
        let (item_kind, role, place) = match kind {
            Kind::Part => (ItemKind::Folder, Role::Part, Place::end_of(Area::Manuscript)),
            Kind::Chapter => {
                (ItemKind::Text, Role::Chapter, self.part.map_or_else(|| Place::end_of(Area::Manuscript), |p| Place::inside(Area::Manuscript, p)))
            }
            Kind::Front => (ItemKind::Text, Role::FrontMatter, Place::end_of(Area::Manuscript)),
            Kind::Back => (ItemKind::Text, Role::BackMatter, Place::end_of(Area::Manuscript)),
        };
        let _ = self.project.binder.insert(place, Item::with_id(id, item_kind, title).role(role));
        if let Some(t) = text {
            self.project.texts.insert(id, ItemText::with_body(t));
        }
        match kind {
            Kind::Part => {
                self.part = Some(id);
                *self.report.made.entry("parts").or_default() += 1;
            }
            Kind::Chapter => *self.report.made.entry("chapters").or_default() += 1,
            Kind::Front => *self.report.made.entry("front matter").or_default() += 1,
            Kind::Back => {
                self.part = None;
                *self.report.made.entry("back matter").or_default() += 1;
            }
        }
        id
    }
}

fn add_counts(into: &mut Dropped, from: Dropped) {
    for (k, v) in from {
        *into.entry(k).or_default() += v;
    }
}

/// Underline stands in for italic in typed manuscripts: shown as italic (IMP-5).
fn underline_as_italic(p: &mut Paragraph) -> usize {
    let mut n = 0;
    for r in &mut p.runs {
        if r.props.underline.is_some() {
            r.props.underline = None;
            r.props.italic = Some(true);
            n += 1;
        }
    }
    n
}

/// The text of a run of paragraphs (with the document's notes). Scene breaks are paragraphs
/// marked `book.block = "break"`, which the conversion makes scene breaks.
fn text_of(doc: &Document, paras: &[Arc<DocBlock>], report: &mut Report) -> Text {
    let mut sub = jcj_doc::new_document();
    sub.parts = doc.parts.clone();
    sub.body = paras.to_vec();
    let (t, dropped) = from_document(&sub);
    add_counts(&mut report.dropped, dropped);
    t
}

/// The words of the document's text in reading order, leaving out what the import leaves out
/// (empty paragraphs, page breaks and scene-break marks, which have no words).
pub fn source_words(doc: &Document) -> Vec<String> {
    // A note reference counts as nothing, as in the book view's plain text.
    doc.body.iter().filter_map(|b| b.as_para()).flat_map(|p| verify::words(&p.text.replace(OBJ, "").replace(PAGE_BREAK, " "))).collect()
}

/// The words of the document's notes, in the order they are referred to.
pub fn source_note_words(doc: &Document) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for b in &doc.body {
        let Some(p) = b.as_para() else { continue };
        for (_, o) in p.objects_with_offsets() {
            if let InlineObject::NoteRef { id, .. } = o
                && seen.insert(*id)
                && let Some(part) = doc.parts.get(id)
            {
                for nb in &part.blocks {
                    if let Some(np) = nb.as_para() {
                        out.extend(verify::words(&np.text.replace(OBJ, "")));
                    }
                }
            }
        }
    }
    out
}

/// The words of an imported project in reading order: each section's title when it came from a
/// heading (not one the import gave it, in `named`), then its text; and, separately, its notes'
/// words.
pub fn imported_words(p: &Project, named: &std::collections::BTreeSet<ItemId>) -> (Vec<String>, Vec<String>) {
    let mut words = Vec::new();
    let mut notes = Vec::new();
    for row in p.binder.rows_of(Area::Manuscript) {
        if !named.contains(&row.item.id) {
            words.extend(verify::words(&row.item.title));
        }
        if let Some(t) = p.text(row.item.id) {
            for b in &t.body.blocks {
                let one = Text { blocks: vec![b.clone()], notes: Vec::new() };
                for para in one.plain_paragraphs() {
                    words.extend(verify::words(&para));
                }
            }
            for n in &t.body.notes {
                for para in &n.paras {
                    notes.extend(verify::words(&wordsmith_manuscript_model::text::plain(para)));
                }
            }
        }
    }
    (words, notes)
}

/// Check an import's words against the document's; refuse it when they differ. `named` holds
/// the sections whose titles the import made up.
pub fn check(doc: &Document, project: &Project, named: &std::collections::BTreeSet<ItemId>, report: &mut Report) -> Result<(), Error> {
    let (words, notes) = imported_words(project, named);
    let w = Check::of(&source_words(doc), &words);
    let n = Check::of(&source_note_words(doc), &notes);
    let failed = if !w.passed() {
        Some(w.to_string())
    } else if !n.passed() {
        Some(format!("in the notes: {n}"))
    } else {
        None
    };
    report.words = Some(w);
    report.notes = Some(n);
    match failed {
        Some(m) => Err(Error::Words(m)),
        None => Ok(()),
    }
}

/// Import a document: detect its structure with `rules`, make the owned source's project titled
/// `title`, and check every word.
pub fn import_document(doc: &Document, rules: &ImportRules, title: &str) -> Result<Imported, Error> {
    import_with(doc, rules, title, &|_| {})
}

/// [`import_document`], with what detection found adjusted by `adjust` before the sections are
/// made (a format that knows its own structure says so).
pub fn import_with(doc: &Document, rules: &ImportRules, title: &str, adjust: &dyn Fn(&mut Vec<Mark>)) -> Result<Imported, Error> {
    let mut doc = doc.clone();
    let mut report = Report::default();
    if !rules.off.iter().any(|r| r == "underline-italic") {
        let mut n = 0;
        for b in &mut doc.body {
            if let DocBlock::Para(p) = Arc::make_mut(b) {
                n += underline_as_italic(p);
            }
        }
        if n > 0 {
            report.changes.insert("underline shown as italic", n);
        }
    }
    let paras = paras_of(&doc);
    let mut marks = detect::detect(&paras, rules);
    adjust(&mut marks);
    let mut b = Builder { project: Project::new(), next: 0, part: None, report, named: std::collections::BTreeSet::new() };
    b.project.settings.book.title = title.to_owned();
    // The section being gathered: its kind and title, and its paragraphs.
    let mut current: Option<(Kind, String)> = None;
    let mut body: Vec<Arc<DocBlock>> = Vec::new();
    let flush = |b: &mut Builder, current: &Option<(Kind, String)>, body: &mut Vec<Arc<DocBlock>>| {
        if current.is_none() && body.is_empty() {
            return;
        }
        let t = text_of(&doc, body, &mut b.report);
        match current {
            Some((Kind::Part, title)) => {
                b.add(Kind::Part, title, if t.is_empty() { None } else { Some(t) });
            }
            Some((kind, title)) => {
                b.add(*kind, title, Some(t));
            }
            // Text before any heading: the title page when it opens the book, else a chapter.
            None => {
                let id = b.add(Kind::Front, "Title Page", Some(t));
                b.named.insert(id);
            }
        }
        body.clear();
    };
    for (i, mark) in marks.iter().enumerate() {
        let Some(block) = doc.body.get(i) else { continue };
        match mark {
            Mark::Heading(kind, rule) => {
                flush(&mut b, &current, &mut body);
                current = Some((*kind, paras[i].text.trim().to_owned()));
                *b.report.kept.entry(if *rule == "heading-styles" { "headings" } else { "headings found by pattern" }).or_default() += 1;
            }
            Mark::Break(_) => {
                let mut empty = Paragraph::new();
                let _ = wordsmith_book_model::document::BLOCK.set(&mut empty, &"break".to_owned());
                body.push(Arc::new(DocBlock::Para(empty)));
                *b.report.made.entry("scene breaks").or_default() += 1;
            }
            Mark::Skip => {
                let what = if paras[i].text.is_empty() && block.as_para().is_some_and(|p| p.text.contains(PAGE_BREAK)) {
                    "page breaks"
                } else {
                    "empty paragraphs"
                };
                *b.report.dropped.entry(what).or_default() += 1;
            }
            Mark::Body => {
                body.push(block.clone());
            }
        }
    }
    flush(&mut b, &current, &mut body);
    let mut report = b.report;
    if !source_note_words(&doc).is_empty() {
        report.kept.insert("notes", doc.parts.len());
    }
    let italic = doc.body.iter().filter_map(|b| b.as_para()).flat_map(|p| p.runs.iter()).filter(|r| r.props.italic == Some(true)).count();
    if italic > 0 {
        report.kept.insert("italic", italic);
    }
    // Scene-break marks are not the author's words (a mark the user chose may hold letters):
    // the check leaves them out on the source's side too.
    let mut checked = doc.clone();
    checked.body = doc.body.iter().zip(&marks).filter(|(_, m)| !matches!(m, Mark::Break(_))).map(|(b, _)| b.clone()).collect();
    check(&checked, &b.project, &b.named, &mut report)?;
    Ok(Imported { project: b.project, report, named: b.named })
}

/// Import a file's bytes. Plain text is read by Book's own reader ([`text`]); every other format
/// through the core.
pub fn import(format: Format, bytes: &[u8], rules: &ImportRules, title: &str) -> Result<Imported, Error> {
    if format == Format::Text {
        let s = std::str::from_utf8(bytes).map_err(|_| Error::Read("the text is not UTF-8".into()))?;
        let doc = text::read(s, rules);
        // The reader keeps every word of the file.
        let check = Check::of(&text::words(s, rules), &verify::document_words(&doc));
        if !check.passed() {
            return Err(Error::Words(check.to_string()));
        }
        return import_document(&doc, rules, title);
    }
    if format == Format::Epub {
        return epub::import(bytes, rules, title);
    }
    let doc = read(format, bytes)?;
    import_document(&doc, rules, title)
}
