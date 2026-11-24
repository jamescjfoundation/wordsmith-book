// SPDX-License-Identifier: MIT OR Apache-2.0
//! Cleanup (IMP-6): typographic changes proposed for an owned source, each listed with its place
//! and shown word by word, accepted or undone one by one or all at once. Straight quotes become
//! curly ones (English), double hyphens em dashes, three periods an ellipsis, and runs of spaces
//! one space, with spaces at the ends of paragraphs removed. Nothing here ever changes a letter,
//! a digit or the order of words; "Keep exactly as written" proposes nothing.

use std::collections::BTreeSet;

use wordsmith_book_model::Cleanup;
use wordsmith_manuscript_model::{Inline, ItemId, Project};

/// A kind of change, by its name in `book.toml`'s `cleanup.off`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// Straight quotes to curly quotes.
    Quotes,
    /// Double hyphens to em dashes.
    Dashes,
    /// Three periods to an ellipsis.
    Ellipses,
    /// Double and trailing spaces.
    Spaces,
}

impl Kind {
    /// Every kind.
    pub const ALL: [Kind; 4] = [Kind::Quotes, Kind::Dashes, Kind::Ellipses, Kind::Spaces];

    /// The kind's name in `book.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Quotes => "quotes",
            Kind::Dashes => "dashes",
            Kind::Ellipses => "ellipses",
            Kind::Spaces => "spaces",
        }
    }

    /// What it does, for the list.
    pub fn title(self) -> &'static str {
        match self {
            Kind::Quotes => "Straight quotes to curly quotes",
            Kind::Dashes => "Double hyphens to dashes",
            Kind::Ellipses => "Three periods to an ellipsis",
            Kind::Spaces => "Double and trailing spaces",
        }
    }
}

/// One proposed change: in section `item`, inline run `run` (in the order of
/// `Text::inline_runs`), span `span`, at byte `at` of the span's text, `from` becomes `to`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// Its number in the list.
    pub id: usize,
    /// The kind.
    pub kind: Kind,
    /// The section.
    pub item: ItemId,
    /// The paragraph, in the order of `Text::inline_runs`.
    pub run: usize,
    /// The span in the paragraph.
    pub span: usize,
    /// The byte offset in the span's text.
    pub at: usize,
    /// What is there.
    pub from: String,
    /// What it becomes.
    pub to: String,
}

/// Words that begin with an apostrophe where letters were left out.
const ELISIONS: [&str; 9] = ["tis", "twas", "twere", "twill", "em", "til", "cause", "n", "bout"];

fn is_open_context(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(c) => c.is_whitespace() || "([{—–-“‘\"'".contains(c),
    }
}

/// The changes one span of text would get.
fn span_changes(text: &str, at_paragraph_start: bool, at_paragraph_end: bool, kinds: &BTreeSet<Kind>, out: &mut Vec<(Kind, usize, String, String)>) {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (pos, c) = chars[i];
        let prev = if i == 0 { None } else { Some(chars[i - 1].1) };
        let prev = if i == 0 && !at_paragraph_start { Some('x') } else { prev };
        let rest = &text[pos..];
        if kinds.contains(&Kind::Ellipses) && (rest.starts_with("...") || rest.starts_with(". . .")) {
            let n = if rest.starts_with("...") { 3 } else { 5 };
            out.push((Kind::Ellipses, pos, rest[..n].to_owned(), "…".into()));
            i += rest[..n].chars().count();
            continue;
        }
        if kinds.contains(&Kind::Dashes) && rest.starts_with("--") {
            let n = if rest.starts_with("---") { 3 } else { 2 };
            out.push((Kind::Dashes, pos, rest[..n].to_owned(), "—".into()));
            i += n;
            continue;
        }
        if kinds.contains(&Kind::Quotes) && c == '"' {
            out.push((Kind::Quotes, pos, "\"".into(), if is_open_context(prev) { "“" } else { "”" }.into()));
        } else if kinds.contains(&Kind::Quotes) && c == '\'' {
            let next = chars.get(i + 1).map(|x| x.1);
            // A word that starts with an elision (’tis, ’twas) takes an apostrophe, not a quote.
            let word: String = text[pos + 1..].chars().take_while(|c| c.is_alphabetic()).collect::<String>().to_lowercase();
            let elision = ELISIONS.contains(&word.as_str());
            let opening = is_open_context(prev) && next.is_some_and(char::is_alphabetic) && !elision;
            out.push((Kind::Quotes, pos, "'".into(), if opening { "‘" } else { "’" }.into()));
        } else if kinds.contains(&Kind::Spaces) && c == ' ' {
            let mut j = i;
            while j < chars.len() && chars[j].1 == ' ' {
                j += 1;
            }
            let end = chars.get(j).map_or(text.len(), |x| x.0);
            let run = &text[pos..end];
            if j == chars.len() && at_paragraph_end {
                out.push((Kind::Spaces, pos, run.to_owned(), String::new()));
            } else if run.len() > 1 {
                out.push((Kind::Spaces, pos, run.to_owned(), " ".into()));
            }
            i = j;
            continue;
        }
        i += 1;
    }
}

/// The kinds a cleanup setting allows.
pub fn kinds(settings: &Cleanup) -> BTreeSet<Kind> {
    if settings.keep_exactly {
        return BTreeSet::new();
    }
    Kind::ALL.into_iter().filter(|k| !settings.off.iter().any(|o| o == k.key())).collect()
}

/// The changes proposed for a project, in reading order.
pub fn propose(p: &Project, settings: &Cleanup) -> Vec<Change> {
    let kinds = kinds(settings);
    let mut out = Vec::new();
    if kinds.is_empty() {
        return out;
    }
    for item in p.included_manuscript() {
        let Some(t) = p.text(item) else { continue };
        for (run, inlines) in t.body.inline_runs().into_iter().enumerate() {
            let last_span = inlines.iter().rposition(|i| matches!(i, Inline::Span { .. }));
            for (span, inline) in inlines.iter().enumerate() {
                let Inline::Span { text, .. } = inline else { continue };
                let mut found = Vec::new();
                span_changes(text, span == 0, Some(span) == last_span, &kinds, &mut found);
                for (kind, at, from, to) in found {
                    out.push(Change { id: out.len(), kind, item, run, span, at, from, to });
                }
            }
        }
    }
    out
}

/// The project with the changes whose ids are in `accepted` made. Changes not accepted (undone)
/// leave the text as it was.
pub fn apply(p: &Project, changes: &[Change], accepted: &BTreeSet<usize>) -> Project {
    let mut out = p.clone();
    let mut chosen: Vec<&Change> = changes.iter().filter(|c| accepted.contains(&c.id)).collect();
    // Later places first, so earlier offsets stay right.
    chosen.sort_by(|a, b| (a.item, a.run, a.span, a.at).cmp(&(b.item, b.run, b.span, b.at)).reverse());
    for c in chosen {
        let Some(t) = out.texts.get_mut(&c.item) else { continue };
        let mut runs = t.body.inline_runs_mut();
        let Some(inlines) = runs.get_mut(c.run) else { continue };
        let Some(Inline::Span { text, .. }) = inlines.get_mut(c.span) else { continue };
        if text.get(c.at..c.at + c.from.len()) == Some(c.from.as_str()) {
            text.replace_range(c.at..c.at + c.from.len(), &c.to);
        }
    }
    for t in out.texts.values_mut() {
        t.body.normalize();
    }
    out
}

/// Every change's id.
pub fn all(changes: &[Change]) -> BTreeSet<usize> {
    changes.iter().map(|c| c.id).collect()
}

/// The paragraph a change is in, before and after it, compared word by word (C2).
pub fn shown(p: &Project, c: &Change) -> Vec<jcj_features::wordiff::Segment> {
    let before = p.text(c.item).and_then(|t| t.body.inline_runs().get(c.run).map(|r| wordsmith_manuscript_model::text::plain(r))).unwrap_or_default();
    let after = apply(p, std::slice::from_ref(c), &BTreeSet::from([c.id]));
    let after =
        after.text(c.item).and_then(|t| t.body.inline_runs().get(c.run).map(|r| wordsmith_manuscript_model::text::plain(r))).unwrap_or_default();
    jcj_features::wordiff::paragraph(&before, &after)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn changes(text: &str) -> String {
        let mut found = Vec::new();
        span_changes(text, true, true, &Kind::ALL.into_iter().collect(), &mut found);
        let mut s = text.to_owned();
        found.sort_by_key(|f| std::cmp::Reverse(f.1));
        for (_, at, from, to) in found {
            s.replace_range(at..at + from.len(), &to);
        }
        s
    }

    #[test]
    fn quotes_dashes_ellipses_and_spaces() {
        assert_eq!(changes("\"Hello,\" she said -- 'twas the end...  Wasn't it? "), "“Hello,” she said — ’twas the end… Wasn’t it?");
        assert_eq!(changes("He said 'yes' and . . . left."), "He said ‘yes’ and … left.");
        assert_eq!(changes("Plain text."), "Plain text.");
    }
}
