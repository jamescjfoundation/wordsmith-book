// SPDX-License-Identifier: MIT OR Apache-2.0
//! The outline (STR-1): front matter, parts, chapters and back matter on the core's tree widget.
//! Book's own elements reorder by drag; the source's sections show in the writer's order,
//! read-only ("Change this in Manuscript").

use jcj_shell::i18n::{tl, tr, tr_with};
use wordsmith_book_model::{BookProject, Element, Placement, SectionId};
use wordsmith_manuscript_format::book::{Book, FrontKind, Section};
use wordsmith_manuscript_model::Role;

use jcj_shell::tree::{TreeRow, Zone};

/// A row of the outline.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    /// A heading: front matter, the body, back matter.
    Heading(Placement, bool),
    /// One of the source's sections.
    Section(SectionId),
    /// One of Book's own elements, by id.
    Element(String),
}

/// The outline key of an assembled piece's id (a section's id, or `element:<id>`).
pub fn key_of_piece(piece: &str) -> Key {
    match piece.strip_prefix("element:") {
        Some(id) => Key::Element(id.to_owned()),
        None => piece.parse::<SectionId>().map_or_else(|_| Key::Element(piece.to_owned()), Key::Section),
    }
}

/// The assembled piece id of an outline key, if it names one.
pub fn piece_of_key(key: &Key) -> Option<String> {
    match key {
        Key::Section(id) => Some(id.to_string()),
        Key::Element(id) => Some(format!("element:{id}")),
        Key::Heading(..) => None,
    }
}

/// What the outline says about a source's section that cannot be moved here.
pub const READ_ONLY: &str = tl("The manuscript's order is the writer's: change this in Manuscript.");

/// A section's label: "Chapter 3", "Part 1: Title", its title, or "Untitled".
pub fn number_label(s: &Section) -> String {
    match (s.role, s.number) {
        (Role::Part, Some(n)) if s.title.trim().is_empty() => tr_with("Part {n}", &[("n", &n.to_string())]),
        (Role::Part, Some(n)) => tr_with("Part {n}: {title}", &[("n", &n.to_string()), ("title", &s.title)]),
        (Role::Chapter, Some(n)) if s.title.trim().is_empty() => tr_with("Chapter {n}", &[("n", &n.to_string())]),
        (Role::Chapter, Some(n)) => format!("{n}. {}", s.title),
        _ if s.title.trim().is_empty() => tr("Untitled"),
        _ => s.title.clone(),
    }
}

fn section_rows(s: &Section, depth: usize, out: &mut Vec<TreeRow<Key>>) {
    let mut r = TreeRow::new(Key::Section(s.id), depth, number_label(s));
    r.draggable = false;
    r.accepts_children = false;
    r.has_children = !s.children.is_empty();
    r.open = true;
    r.glyph = match s.role {
        Role::Part => "§".into(),
        Role::Chapter => "¶".into(),
        _ => "·".into(),
    };
    out.push(r);
    for c in &s.children {
        section_rows(c, depth + 1, out);
    }
}

fn element_row(e: &Element) -> TreeRow<Key> {
    let title = if e.title.is_empty() { tr(e.kind.map_or("Untitled", |k| k.title())) } else { e.title.clone() };
    let mut r = TreeRow::new(Key::Element(e.id.clone()), 1, title);
    r.accepts_children = false;
    r.glyph = "+".into();
    r.trailing = tr("Book");
    r
}

fn heading(placement: Placement, body: bool, label: &str) -> TreeRow<Key> {
    let mut r = TreeRow::new(Key::Heading(placement, body), 0, label);
    r.heading = true;
    r.draggable = false;
    r.has_children = true;
    r.open = true;
    r
}

/// The outline's rows for a project and its source's book view (none when the source did not
/// open: Book's own elements still show).
pub fn rows(project: &BookProject, book: Option<&Book>) -> Vec<TreeRow<Key>> {
    let mut out = vec![heading(Placement::Front, false, &tr("Front matter"))];
    for e in project.elements.iter().filter(|e| e.placement == Placement::Front) {
        out.push(element_row(e));
    }
    if let Some(b) = book {
        for s in &b.front {
            section_rows(s, 1, &mut out);
        }
    }
    out.push(heading(Placement::Front, true, &tr("Body")));
    if let Some(b) = book {
        for s in &b.body {
            section_rows(s, 1, &mut out);
        }
    }
    out.push(heading(Placement::Back, false, &tr("Back matter")));
    if let Some(b) = book {
        for s in &b.back {
            section_rows(s, 1, &mut out);
        }
    }
    for e in project.elements.iter().filter(|e| e.placement == Placement::Back) {
        out.push(element_row(e));
    }
    out
}

/// Move Book's element `id` before, after or inside (a heading) `target`. Sections of the source
/// cannot be moved, and nothing moves among them.
pub fn move_element(project: &mut BookProject, id: &str, target: &Key, zone: Zone) -> Result<(), String> {
    let from = project.elements.iter().position(|e| e.id == id).ok_or_else(|| format!("there is no element `{id}`"))?;
    match target {
        Key::Section(_) => Err(READ_ONLY.into()),
        Key::Heading(_, true) => Err("Book's elements go in the front or back matter, not the body.".into()),
        Key::Heading(placement, false) => {
            let mut e = project.elements.remove(from);
            e.placement = *placement;
            // Inside a heading: first of front matter, last of back matter.
            match placement {
                Placement::Front => project.elements.insert(0, e),
                Placement::Back => project.elements.push(e),
            }
            Ok(())
        }
        Key::Element(other) => {
            if other == id {
                return Ok(());
            }
            let mut e = project.elements.remove(from);
            let to = project.elements.iter().position(|x| &x.id == other).ok_or_else(|| format!("there is no element `{other}`"))?;
            e.placement = project.elements[to].placement;
            let at = if zone == Zone::After { to + 1 } else { to };
            project.elements.insert(at, e);
            Ok(())
        }
    }
}

/// A front matter kind's name, for the source's own front and back matter.
pub fn kind_title(k: FrontKind) -> &'static str {
    match k {
        FrontKind::TitlePage => tl("Title page"),
        FrontKind::Copyright => tl("Copyright"),
        FrontKind::Dedication => tl("Dedication"),
        FrontKind::Epigraph => tl("Epigraph"),
        FrontKind::Contents => tl("Contents"),
        FrontKind::Acknowledgements => tl("Acknowledgements"),
        FrontKind::AboutTheAuthor => tl("About the author"),
        FrontKind::AlsoBy => tl("Also by"),
        FrontKind::Other => tl("Other"),
    }
}
