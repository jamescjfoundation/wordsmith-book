// SPDX-License-Identifier: MIT OR Apache-2.0
//! Light fixes (STR-7, plan section 6.4): exact words within one paragraph of a section become
//! others. In a book's own source a fix is made in the text. For a linked Manuscript project it
//! is offered to the writer in the offers file beside the project (Manuscript's `offers` module,
//! appended, never rewritten), applied in Book's builds while it waits, and followed through the
//! writer's decisions: accepted and in the text, it retires; rejected, it leaves the builds and
//! the user is told; words no longer there exactly once, it is stale. Never a silent overwrite.

use std::path::Path;

use wordsmith_book_model::{Fix, SectionId};
use wordsmith_manuscript_format::book::{Book, Section};
use wordsmith_manuscript_format::offers::{self, Decision, Offer, OffersFile};
use wordsmith_manuscript_model::{Inline, Inlines, Text};

use crate::Error;

/// Where a fix stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// Made in the book's own source.
    Made,
    /// Offered to the writer, no decision yet: applied in Book's builds.
    Waiting,
    /// The writer took it and the text has it: the fix retires.
    Accepted,
    /// The writer turned it down: left out of builds.
    Rejected,
    /// The words are no longer there exactly once: shown with the paragraph as it is now.
    Stale(String),
}

impl State {
    /// What the state means, in plain words.
    pub fn describe(&self) -> String {
        match self {
            State::Made => "Made in the book's own text.".into(),
            State::Waiting => "Not yet accepted by the writer; Book's builds use it meanwhile.".into(),
            State::Accepted => "Accepted by the writer and now in the manuscript.".into(),
            State::Rejected => "Turned down by the writer; Book's builds leave it out.".into(),
            State::Stale(now) => format!("The words have changed in the manuscript; the paragraph now reads: “{now}”."),
        }
    }
}

fn find_section(sections: &[Section], id: SectionId) -> Option<&Section> {
    for s in sections {
        if s.id == id {
            return Some(s);
        }
        if let Some(f) = find_section(&s.children, id) {
            return Some(f);
        }
    }
    None
}

/// The section `id` of a book view.
pub fn section(b: &Book, id: SectionId) -> Option<&Section> {
    find_section(&b.front, id).or_else(|| find_section(&b.body, id)).or_else(|| find_section(&b.back, id))
}

/// How often `find` occurs in the paragraphs of a text, and the paragraph of the first.
fn occurrences(t: &Text, find: &str) -> (usize, Option<String>) {
    let mut n = 0;
    let mut first = None;
    for p in t.inline_runs() {
        let plain = wordsmith_manuscript_model::text::plain(p);
        let c = plain.matches(find).count();
        if c > 0 && first.is_none() {
            first = Some(plain.clone());
        }
        n += c;
    }
    (n, first)
}

/// Replace `find` (exactly once in the text, within one paragraph) with `replace`, keeping the
/// marks: the new words take the marks of the first word they replace.
pub fn apply(t: &mut Text, find: &str, replace: &str) -> Result<(), String> {
    if find.is_empty() || find.contains('\n') || replace.contains('\n') {
        return Err("A fix changes words within one paragraph.".into());
    }
    match occurrences(t, find).0 {
        0 => return Err(format!("“{find}” is not in the section.")),
        1 => {}
        n => return Err(format!("“{find}” is in the section {n} times; give a few more words so it is there once.")),
    }
    for p in t.inline_runs_mut() {
        let plain = wordsmith_manuscript_model::text::plain(p);
        let Some(start) = plain.find(find) else { continue };
        replace_in(p, start, start + find.len(), replace);
        break;
    }
    t.normalize();
    Ok(())
}

/// Replace the plain-text bytes `start..end` of a paragraph with `with`.
fn replace_in(p: &mut Inlines, start: usize, end: usize, with: &str) {
    let mut at = 0;
    let mut placed = false;
    for inline in p.iter_mut() {
        let Inline::Span { text, .. } = inline else { continue };
        let (s0, s1) = (at, at + text.len());
        at = s1;
        if s1 <= start || s0 >= end {
            continue;
        }
        let a = start.max(s0) - s0;
        let b = end.min(s1) - s0;
        let insert = if placed { "" } else { with };
        text.replace_range(a..b, insert);
        placed = true;
    }
}

/// Where a fix stands, from the writer's decisions and the section's text now.
pub fn state(fix: &Fix, linked: bool, decision: Option<Decision>, now: Option<&Section>) -> State {
    if !linked {
        return State::Made;
    }
    let text = now.map(|s| Text { blocks: s.content.clone(), notes: s.notes.clone() }).unwrap_or_default();
    let (found, para) = occurrences(&text, &fix.find);
    match decision {
        Some(Decision::Rejected) => State::Rejected,
        Some(Decision::Accepted) if occurrences(&text, &fix.replace).0 >= 1 && found == 0 => State::Accepted,
        Some(Decision::Accepted) => State::Stale(para.unwrap_or_default()),
        None if found == 1 => State::Waiting,
        None => {
            let shown = para.or_else(|| text.plain_paragraphs().into_iter().next()).unwrap_or_default();
            State::Stale(shown)
        }
    }
}

/// The book view with the fixes that are waiting applied (Book's builds use them meanwhile).
pub fn apply_waiting(b: &mut Book, fixes: &[Fix], states: &[State]) {
    for (fix, st) in fixes.iter().zip(states) {
        if *st != State::Waiting {
            continue;
        }
        let Some(id) = fix.section else { continue };
        let mut stack: Vec<&mut Section> = b.front.iter_mut().chain(b.body.iter_mut()).chain(b.back.iter_mut()).collect();
        while let Some(s) = stack.pop() {
            if s.id == id {
                let mut t = Text { blocks: std::mem::take(&mut s.content), notes: std::mem::take(&mut s.notes) };
                let _ = apply(&mut t, &fix.find, &fix.replace);
                s.content = t.blocks;
                s.notes = t.notes;
                break;
            }
            stack.extend(s.children.iter_mut());
        }
    }
}

/// Append an offer to the offers file beside the Manuscript project at `root`. Offers already in
/// the file are kept exactly as they are; a newer or damaged file is never written over.
pub fn offer(root: &Path, fix: &Fix) -> Result<(), Error> {
    let section = fix.section.ok_or_else(|| Error::Write("a fix needs its section".into()))?;
    let path = offers::offers_path(root);
    let mut file = if path.exists() {
        let bytes = jcj_project::files::read_bounded(&path, offers::MAX_FILE)
            .map_err(|e| Error::Io { file: path.display().to_string(), why: e.to_string() })?;
        offers::read(&bytes).map_err(|why| Error::Damaged { file: path.display().to_string(), why })?
    } else {
        OffersFile { version: offers::VERSION, offers: Vec::new() }
    };
    if file.offers.iter().any(|o| o.id == fix.id) {
        return Err(Error::Write(format!("the offer `{}` is already in the offers file", fix.id)));
    }
    file.offers.push(Offer {
        id: fix.id.clone(),
        item: section,
        find: fix.find.clone(),
        replace: fix.replace.clone(),
        note: fix.note.clone(),
        made: fix.made.clone(),
    });
    let bytes = offers::write(&file).map_err(Error::Write)?;
    jcj_project::files::write_atomic(&path, &bytes).map_err(|e| Error::Io { file: path.display().to_string(), why: e.to_string() })
}

/// The smallest fix that turns paragraph `old` into `new`: the changed words with enough words
/// around them to be found once in `section` (a light fix made by editing a paragraph).
pub fn from_edit(old: &str, new: &str, section: &Text) -> Option<(String, String)> {
    if old == new {
        return None;
    }
    let a: Vec<&str> = old.split_inclusive(' ').collect();
    let b: Vec<&str> = new.split_inclusive(' ').collect();
    let pre = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suf = a.iter().rev().zip(b.iter().rev()).take_while(|(x, y)| x == y).count().min(a.len() - pre).min(b.len() - pre);
    // Widen the changed words by one word on each side until they are found once.
    let (mut lo, mut hi) = (pre, suf);
    loop {
        let mut find: String = a[lo..a.len() - hi].concat();
        let mut replace: String = b[lo..b.len() - hi].concat();
        if find.ends_with(' ') && replace.ends_with(' ') {
            find.pop();
            replace.pop();
        }
        if !find.trim().is_empty() && occurrences(section, &find).0 == 1 {
            return Some((find, replace));
        }
        if lo == 0 && hi == 0 {
            return None;
        }
        lo = lo.saturating_sub(1);
        hi = hi.saturating_sub(1);
    }
}

/// Each fix's state for an opened source.
pub fn states(fixes: &[Fix], source: &crate::source::Opened) -> Vec<State> {
    let linked = source.kind != crate::source::Kind::Owned;
    let decisions = offers::decisions(&source.project);
    fixes.iter().map(|f| state(f, linked, decisions.get(&f.id).copied(), f.section.and_then(|id| section(&source.book, id)))).collect()
}

/// The book view a build uses: the source's, with the fixes that wait on the writer applied.
pub fn book_for_build(fixes: &[Fix], source: &crate::source::Opened) -> Book {
    let mut b = source.book.clone();
    if source.kind != crate::source::Kind::Owned {
        apply_waiting(&mut b, fixes, &states(fixes, source));
    }
    b
}
