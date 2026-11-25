// SPDX-License-Identifier: MIT OR Apache-2.0
//! The book as print and ebook build it: front matter, body and back matter as pieces in reading
//! order, from the source's book view and Book's own layer. Book's generated elements go beside
//! the source's own front and back matter; where the source already has one of a kind (its own
//! copyright page, say), the source's is used and Book's is left out, never doubled.

use wordsmith_book_model::matter::{self, Meta};
use wordsmith_book_model::{BookProject, Edition, ElementKind, Only, Placement, SectionId};
use wordsmith_manuscript_format::book::{Book, FrontKind, Section};
use wordsmith_manuscript_model::{Block, Note, Role};

/// What a piece of the book is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum What {
    /// One of Book's own elements.
    Element(ElementKind),
    /// The source's front or back matter, of a kind.
    Matter(FrontKind),
    /// A part page.
    Part,
    /// A chapter.
    Chapter,
    /// A plain section of the source with no role.
    Other,
}

/// One piece of the book: a page or run of pages that opens on its own.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    /// A stable id: the section's id, or `element:<id>`.
    pub id: String,
    /// What it is.
    pub what: What,
    /// The source section it comes from.
    pub section: Option<SectionId>,
    /// Its number (parts and chapters, each counting on its own).
    pub number: Option<u32>,
    /// Its number line as printed ("Chapter One"), from the book's numbering; none when it is
    /// unnumbered, the style is none, or its title is already a number line.
    pub label: Option<String>,
    /// Its title.
    pub title: String,
    /// A subtitle (Book's setting).
    pub subtitle: String,
    /// An epigraph and its attribution (Book's setting).
    pub epigraph: Option<(String, String)>,
    /// Its text.
    pub blocks: Vec<Block>,
    /// Its notes.
    pub notes: Vec<Note>,
    /// Which outputs show it.
    pub only: Only,
    /// Which outputs show each of its blocks (as long as `blocks`).
    pub block_only: Vec<Only>,
    /// Hide its number.
    pub unnumbered: bool,
}

/// The assembled book.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Assembled {
    /// The metadata front and back matter were made from.
    pub meta: Meta,
    /// Front matter.
    pub front: Vec<Piece>,
    /// Parts and chapters, in reading order (a part's chapters follow it).
    pub body: Vec<Piece>,
    /// Back matter.
    pub back: Vec<Piece>,
    /// Placeholders the element templates did not know, for preflight.
    pub unknown: Vec<String>,
}

impl Assembled {
    /// Every piece in reading order.
    pub fn pieces(&self) -> impl Iterator<Item = &Piece> {
        self.front.iter().chain(&self.body).chain(&self.back)
    }
}

/// A section's own text and its children's, as one run of blocks: each child after a scene break
/// when it is a scene (the writer's scenes make the chapter).
fn flatten(s: &Section, blocks: &mut Vec<Block>, notes: &mut Vec<Note>) {
    blocks.extend(s.content.iter().cloned());
    notes.extend(s.notes.iter().cloned());
    for (i, c) in s.children.iter().enumerate() {
        if c.role == Role::Scene && (i > 0 || !blocks.is_empty()) && !blocks.is_empty() {
            blocks.push(Block::SceneBreak);
        }
        flatten(c, blocks, notes);
    }
}

fn piece(project: &BookProject, s: &Section, what: What) -> Piece {
    let settings = project.section(s.id);
    let mut blocks = Vec::new();
    let mut notes = Vec::new();
    if matches!(what, What::Part) {
        blocks.extend(s.content.iter().cloned());
        notes.extend(s.notes.iter().cloned());
    } else {
        flatten(s, &mut blocks, &mut notes);
    }
    Piece {
        id: s.id.as_text(),
        what,
        section: Some(s.id),
        number: s.number,
        label: None,
        title: s.title.clone(),
        subtitle: settings.subtitle,
        epigraph: (!settings.epigraph.is_empty()).then_some((settings.epigraph, settings.epigraph_source)),
        block_only: blocks.iter().map(|b| wordsmith_book_model::BlockOnly::of(&settings.blocks_only, &block_text(b))).collect(),
        blocks,
        notes,
        only: settings.only,
        unnumbered: settings.unnumbered,
    }
}

/// A block's plain text (for finding a paragraph by its first words).
pub fn block_text(b: &Block) -> String {
    use wordsmith_manuscript_model::text::plain;
    match b {
        Block::Para(c) | Block::Heading { content: c, .. } => plain(c),
        Block::Quote(ps) | Block::Fenced { paras: ps, .. } => ps.iter().map(|p| plain(p)).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

impl Piece {
    /// Its blocks as one kind of output shows them (print, or the ebook).
    pub fn blocks_for(&self, print: bool) -> Vec<Block> {
        let hidden = if print { Only::Ebook } else { Only::Print };
        self.blocks
            .iter()
            .zip(self.block_only.iter().chain(std::iter::repeat(&Only::Both)))
            .filter(|(_, o)| **o != hidden)
            .map(|(b, _)| b.clone())
            .collect()
    }
}

/// The book view as one kind of output shows it: sections and paragraphs set to show only in
/// the other left out (STR-9). The word checks compare each output with this.
pub fn shown_in(project: &BookProject, book: &Book, print: bool) -> Book {
    let hidden = if print { Only::Ebook } else { Only::Print };
    fn walk(project: &BookProject, s: &Section, rules: &[wordsmith_book_model::BlockOnly], hidden: Only) -> Option<Section> {
        let settings = project.section(s.id);
        if settings.only == hidden {
            return None;
        }
        let mut rules = rules.to_vec();
        rules.extend(settings.blocks_only);
        let mut out = s.clone();
        out.content.retain(|b| wordsmith_book_model::BlockOnly::of(&rules, &block_text(b)) != hidden);
        out.children = s.children.iter().filter_map(|c| walk(project, c, &rules, hidden)).collect();
        Some(out)
    }
    let mut out = book.clone();
    for part in [&mut out.front, &mut out.body, &mut out.back] {
        *part = part.iter().filter_map(|s| walk(project, s, &[], hidden)).collect();
    }
    out
}

/// The front matter kind an element stands for in the source (the source's own wins).
fn source_kind(k: ElementKind) -> Option<FrontKind> {
    Some(match k {
        ElementKind::TitlePage => FrontKind::TitlePage,
        ElementKind::Copyright => FrontKind::Copyright,
        ElementKind::Dedication => FrontKind::Dedication,
        ElementKind::Epigraph => FrontKind::Epigraph,
        ElementKind::Contents => FrontKind::Contents,
        ElementKind::Acknowledgements => FrontKind::Acknowledgements,
        ElementKind::AboutTheAuthor => FrontKind::AboutTheAuthor,
        ElementKind::AlsoBy => FrontKind::AlsoBy,
        _ => return None,
    })
}

fn body(project: &BookProject, sections: &[Section], out: &mut Vec<Piece>) {
    for s in sections {
        match s.role {
            Role::Part => {
                out.push(piece(project, s, What::Part));
                body(project, &s.children, out);
            }
            Role::Chapter => out.push(piece(project, s, What::Chapter)),
            // A plain folder holding chapters is not a page of its own.
            _ if s.children.iter().any(|c| matches!(c.role, Role::Chapter | Role::Part)) => body(project, &s.children, out),
            _ => out.push(piece(project, s, What::Other)),
        }
    }
}

/// Assemble the book for `edition` (or the book as a whole).
pub fn assemble(project: &BookProject, book: &Book, edition: Option<&Edition>) -> Assembled {
    let meta = matter::meta(&book.metadata.info, &book.metadata.byline, &project.metadata, edition);
    let mut a = Assembled { meta, ..Assembled::default() };
    // A Notes element with no text of its own holds the book's notes when they go at its end.
    fn any_notes(s: &[Section]) -> bool {
        s.iter().any(|x| !x.notes.is_empty() || any_notes(&x.children))
    }
    let notes_at_end = edition.is_none_or(|e| e.notes == wordsmith_book_model::NoteStyle::BookEnd)
        && (any_notes(&book.front) || any_notes(&book.body) || any_notes(&book.back));
    for placement in [Placement::Front, Placement::Back] {
        let source: &[Section] = if placement == Placement::Front { &book.front } else { &book.back };
        let mut used = vec![false; source.len()];
        let mut pieces = Vec::new();
        for e in project.elements.iter().filter(|e| e.placement == placement) {
            let Some(kind) = e.kind else { continue };
            // An element for some stores shows only in editions for one of them (STR-10).
            if !e.stores.is_empty() && !edition.is_some_and(|ed| e.stores.iter().any(|s| s.trim().eq_ignore_ascii_case(ed.store.trim()))) {
                continue;
            }
            // The source's own page of this kind goes here instead of Book's.
            if let Some(fk) = source_kind(kind)
                && let Some(i) = source.iter().position(|s| s.kind == fk)
            {
                if !used[i] {
                    used[i] = true;
                    pieces.push(piece(project, &source[i], What::Matter(fk)));
                }
                continue;
            }
            let (lines, unknown) = matter::lines(kind, &e.text, &a.meta);
            a.unknown.extend(unknown);
            // A generated element with nothing to say (no biography, no other books) is left out.
            if lines.is_empty() && kind != ElementKind::Contents && !(kind == ElementKind::Endnotes && notes_at_end) {
                continue;
            }
            pieces.push(Piece {
                id: format!("element:{}", e.id),
                what: What::Element(kind),
                section: None,
                number: None,
                label: None,
                title: if e.title.is_empty() { kind.title().to_owned() } else { e.title.clone() },
                subtitle: String::new(),
                epigraph: None,
                blocks: matter::blocks(&lines),
                notes: Vec::new(),
                block_only: Vec::new(),
                only: e.only,
                unnumbered: true,
            });
        }
        let rest = source.iter().enumerate().filter(|(i, _)| !used[*i]).map(|(_, s)| piece(project, s, What::Matter(s.kind)));
        if placement == Placement::Front {
            pieces.extend(rest);
            a.front = pieces;
        } else {
            let mut back: Vec<Piece> = rest.collect();
            back.extend(pieces);
            a.back = back;
        }
    }
    body(project, &book.body, &mut a.body);
    for p in &mut a.body {
        p.label = number_line(project, p);
    }
    a
}

/// A part's or chapter's number line under the book's numbering.
pub fn number_line(project: &BookProject, p: &Piece) -> Option<String> {
    let n = p.number?;
    if p.unnumbered || wordsmith_book_model::numbers::is_number_line(&p.title) {
        return None;
    }
    let num = &project.numbering;
    match p.what {
        What::Part => wordsmith_book_model::numbers::label(num.parts, n, if num.part_label.is_empty() { "Part" } else { &num.part_label }),
        What::Chapter => {
            wordsmith_book_model::numbers::label(num.chapters, n, if num.chapter_label.is_empty() { "Chapter" } else { &num.chapter_label })
        }
        _ => None,
    }
}
