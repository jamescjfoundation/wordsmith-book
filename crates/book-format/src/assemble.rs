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
        title: s.title.clone(),
        subtitle: settings.subtitle,
        epigraph: (!settings.epigraph.is_empty()).then_some((settings.epigraph, settings.epigraph_source)),
        blocks,
        notes,
        only: settings.only,
        unnumbered: settings.unnumbered,
    }
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
    for placement in [Placement::Front, Placement::Back] {
        let source: &[Section] = if placement == Placement::Front { &book.front } else { &book.back };
        let mut used = vec![false; source.len()];
        let mut pieces = Vec::new();
        for e in project.elements.iter().filter(|e| e.placement == placement) {
            let Some(kind) = e.kind else { continue };
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
            if lines.is_empty() && kind != ElementKind::Contents {
                continue;
            }
            pieces.push(Piece {
                id: format!("element:{}", e.id),
                what: What::Element(kind),
                section: None,
                number: None,
                title: if e.title.is_empty() { kind.title().to_owned() } else { e.title.clone() },
                subtitle: String::new(),
                epigraph: None,
                blocks: matter::blocks(&lines),
                notes: Vec::new(),
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
    a
}
