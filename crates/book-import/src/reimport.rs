// SPDX-License-Identifier: MIT OR Apache-2.0
//! Re-import (IMP-8): a new version of the file a book was imported from, compared with the
//! book's own source section by section and word by word, so the writer sees every difference
//! before the new text replaces the old. Book's layer (design, metadata, front and back matter,
//! each section's settings) is kept: sections are matched by their role and title, in order.

use std::collections::BTreeMap;

use jcj_features::wordiff::{self, Change, Diff};
use wordsmith_manuscript_format::book::{Book, Section};
use wordsmith_manuscript_model::{ItemId, Role, Text};

/// One section's differences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionChange {
    /// Its title (the new version's, else the old one's).
    pub title: String,
    /// The section in the book now, if it was there.
    pub old: Option<ItemId>,
    /// The section in the new version, if it is there.
    pub new: Option<ItemId>,
    /// Its paragraphs compared word by word.
    pub diff: Diff,
}

impl SectionChange {
    /// What happened to it, in a word.
    pub fn status(&self) -> &'static str {
        match (self.old, self.new) {
            (Some(_), None) => "removed",
            (None, Some(_)) => "added",
            _ => "changed",
        }
    }
}

/// A comparison of the book's source with a new version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Comparison {
    /// The sections that differ, in reading order.
    pub changes: Vec<SectionChange>,
    /// Each section of the book now and its match in the new version.
    pub matched: BTreeMap<ItemId, ItemId>,
    /// Words only in the new version.
    pub added: usize,
    /// Words only in the book now.
    pub removed: usize,
}

impl Comparison {
    /// Whether the new version holds the same words.
    pub fn is_same(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Every section in reading order (parts, chapters, scenes, front and back matter).
fn flatten(b: &Book) -> Vec<&Section> {
    fn walk<'a>(s: &'a Section, out: &mut Vec<&'a Section>) {
        out.push(s);
        for c in &s.children {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    for s in b.front.iter().chain(&b.body).chain(&b.back) {
        walk(s, &mut out);
    }
    out
}

fn key(s: &Section) -> (String, String) {
    let role = match s.role {
        Role::Part => "part",
        Role::Chapter => "chapter",
        Role::Scene => "scene",
        _ => "other",
    };
    (role.to_owned(), s.title.trim().to_lowercase())
}

fn paras(s: &Section) -> Vec<String> {
    Text { blocks: s.content.clone(), notes: s.notes.clone() }.plain_paragraphs()
}

/// The longest run of sections the two versions have in common, by role and title, as index
/// pairs.
fn common(a: &[(String, String)], b: &[(String, String)]) -> Vec<(usize, usize)> {
    let (n, m) = (a.len(), b.len());
    // Bounded: a book has at most a few thousand sections.
    if n * m > 25_000_000 {
        return (0..n.min(m)).map(|i| (i, i)).collect();
    }
    let mut t = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i][j] = if a[i] == b[j] { t[i + 1][j + 1] + 1 } else { t[i + 1][j].max(t[i][j + 1]) };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n && j < m {
        if a[i] == b[j] {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if t[i + 1][j] >= t[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// Compare the book's source with a new version of it.
pub fn compare(old: &Book, new: &Book) -> Comparison {
    let (a, b) = (flatten(old), flatten(new));
    let (ka, kb): (Vec<_>, Vec<_>) = (a.iter().map(|s| key(s)).collect(), b.iter().map(|s| key(s)).collect());
    let pairs = common(&ka, &kb);
    let mut out = Comparison::default();
    let (mut i, mut j) = (0, 0);
    let push = |c: SectionChange, out: &mut Comparison| {
        out.added += c.diff.added;
        out.removed += c.diff.removed;
        if !c.diff.is_same() || c.old.is_none() || c.new.is_none() {
            out.changes.push(c);
        }
    };
    for (pi, pj) in pairs.into_iter().chain(std::iter::once((a.len(), b.len()))) {
        // Sections only in one version, between matches; where both sides have one, they are the
        // same section renamed.
        while i < pi || j < pj {
            let (o, n) = (a.get(i).filter(|_| i < pi), b.get(j).filter(|_| j < pj));
            let diff = wordiff::paragraphs(&o.map(|s| paras(s)).unwrap_or_default(), &n.map(|s| paras(s)).unwrap_or_default());
            if let (Some(o), Some(n)) = (o, n) {
                out.matched.insert(o.id, n.id);
            }
            let title = n.or(o).map(|s| s.title.clone()).unwrap_or_default();
            push(SectionChange { title, old: o.map(|s| s.id), new: n.map(|s| s.id), diff }, &mut out);
            if i < pi {
                i += 1;
            }
            if j < pj {
                j += 1;
            }
        }
        if pi < a.len() && pj < b.len() {
            let (o, n) = (a[pi], b[pj]);
            out.matched.insert(o.id, n.id);
            let diff = wordiff::paragraphs(&paras(o), &paras(n));
            push(SectionChange { title: n.title.clone(), old: Some(o.id), new: Some(n.id), diff }, &mut out);
        }
        i = pi + 1;
        j = pj + 1;
    }
    out
}

/// A section's changes as lines for a report: "+ words" added, "- words" removed, in order.
pub fn lines(c: &SectionChange) -> Vec<String> {
    let mut out = Vec::new();
    for para in &c.diff.paras {
        for s in para {
            match s.change {
                Change::Added => out.push(format!("+ {}", s.text.trim())),
                Change::Removed => out.push(format!("- {}", s.text.trim())),
                Change::Same => {}
            }
        }
    }
    out
}
