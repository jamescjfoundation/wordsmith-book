// SPDX-License-Identifier: MIT OR Apache-2.0
//! No-break text and hyphenation hints (TXT-5), per word: the book's text as it is set, with
//! the spaces inside words and phrases kept together made no-break spaces, and the words the
//! writer showed how to break given soft hyphens at those places (and only there). The source is
//! never changed; the hints are applied when the book is assembled, for print and ebook alike.
//!
//! Matching is by word, ignoring case and any soft hyphens already in the text, within one run
//! of equally styled text.

use std::collections::HashMap;

use wordsmith_book_model::Hints;
use wordsmith_manuscript_model::{Inline, Text};

/// The soft hyphen: a place a word may break, shown only when it does.
pub const SOFT_HYPHEN: char = '\u{AD}';
/// The no-break space.
pub const NO_BREAK_SPACE: char = '\u{A0}';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Word,
    Space,
    Other,
}

/// A piece of text: a word (letters, digits, apostrophes and soft hyphens), a run of spaces, or
/// one other character.
#[derive(Clone, Debug)]
struct Token {
    kind: Kind,
    text: String,
}

impl Token {
    /// What it is matched by: a word without its soft hyphens, in lower case; any run of spaces
    /// alike.
    fn key(&self) -> String {
        match self.kind {
            Kind::Word => self.text.chars().filter(|c| *c != SOFT_HYPHEN).flat_map(char::to_lowercase).collect(),
            Kind::Space => " ".into(),
            Kind::Other => self.text.clone(),
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == SOFT_HYPHEN || c == '\'' || c == '\u{2019}'
}

fn tokens(s: &str) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::new();
    for c in s.chars() {
        let kind = if is_word_char(c) {
            Kind::Word
        } else if c.is_whitespace() {
            Kind::Space
        } else {
            Kind::Other
        };
        match out.last_mut() {
            Some(t) if t.kind == kind && kind != Kind::Other => t.text.push(c),
            _ => out.push(Token { kind, text: c.to_string() }),
        }
    }
    out
}

/// The hints, ready to apply.
#[derive(Clone, Debug, Default)]
pub struct Prepared {
    /// Each word's break places (char positions after which it may break), by its key.
    breaks: HashMap<String, Vec<usize>>,
    /// Each phrase kept together, as token keys, by its first key.
    keep: HashMap<String, Vec<Vec<String>>>,
}

impl Prepared {
    /// Whether there is nothing to apply.
    pub fn is_empty(&self) -> bool {
        self.breaks.is_empty() && self.keep.is_empty()
    }
}

/// The break places of a hyphenation hint ("man-u-script"): the word and the char positions
/// after which it may break. `None` unless it is one word.
fn breaks_of(entry: &str) -> Option<(String, Vec<usize>)> {
    let entry = entry.trim();
    let mut word = String::new();
    let mut at = Vec::new();
    for c in entry.chars() {
        if c == '-' {
            let n = word.chars().count();
            if n == 0 || at.last() == Some(&n) {
                return None;
            }
            at.push(n);
        } else {
            word.push(c);
        }
    }
    let t = tokens(&word);
    if t.len() != 1 || t[0].kind != Kind::Word || at.last() == Some(&word.chars().count()) {
        return None;
    }
    Some((t[0].key(), at))
}

/// Prepare a project's hints (entries that cannot be used are left out; see
/// [`Hints::problems`]).
pub fn prepare(h: &Hints) -> Prepared {
    let mut p = Prepared::default();
    for e in h.hyphenate.iter().take(wordsmith_book_model::MAX_HINTS) {
        if let Some((key, at)) = breaks_of(e) {
            p.breaks.insert(key, at);
        }
    }
    for e in h.keep.iter().take(wordsmith_book_model::MAX_HINTS) {
        let keys: Vec<String> = tokens(e.trim()).iter().map(Token::key).collect();
        if let Some(first) = keys.first().filter(|_| keys.iter().any(|k| k != " ")) {
            p.keep.entry(first.clone()).or_default().push(keys);
        }
    }
    p
}

/// A word with soft hyphens only at the hint's places.
fn hyphenated(word: &str, at: &[usize]) -> String {
    let mut out = String::new();
    for (i, c) in word.chars().filter(|c| *c != SOFT_HYPHEN).enumerate() {
        if i > 0 && at.contains(&i) {
            out.push(SOFT_HYPHEN);
        }
        out.push(c);
    }
    out
}

/// A run of text with the hints applied.
pub fn apply_str(p: &Prepared, s: &str) -> String {
    if p.is_empty() {
        return s.to_owned();
    }
    let mut t = tokens(s);
    let keys: Vec<String> = t.iter().map(Token::key).collect();
    // Break places first: a word's own soft hyphens give way to the hint's.
    for (tok, key) in t.iter_mut().zip(&keys) {
        if tok.kind == Kind::Word
            && let Some(at) = p.breaks.get(key)
        {
            tok.text = hyphenated(&tok.text, at);
        }
    }
    // Then phrases kept together: whole words only, their spaces no-break and their words not
    // hyphenated.
    let mut i = 0;
    while i < t.len() {
        let found = p
            .keep
            .get(&keys[i])
            .and_then(|phrases| phrases.iter().filter(|ph| keys.get(i..i + ph.len()).is_some_and(|k| k == ph.as_slice())).map(Vec::len).max());
        let whole = |n: usize| {
            let before = i == 0 || t[i - 1].kind != Kind::Word || t[i].kind != Kind::Word;
            let after = t.get(i + n).is_none_or(|x| x.kind != Kind::Word || t[i + n - 1].kind != Kind::Word);
            before && after
        };
        match found.filter(|n| whole(*n)) {
            Some(n) => {
                for tok in &mut t[i..i + n] {
                    match tok.kind {
                        Kind::Space => tok.text = NO_BREAK_SPACE.to_string(),
                        Kind::Word => tok.text.retain(|c| c != SOFT_HYPHEN),
                        Kind::Other => {}
                    }
                }
                i += n;
            }
            None => i += 1,
        }
    }
    t.into_iter().map(|x| x.text).collect()
}

/// A text with the hints applied, its notes too.
pub fn apply(p: &Prepared, text: &mut Text) {
    if p.is_empty() {
        return;
    }
    for run in text.inline_runs_mut() {
        for inline in run.iter_mut() {
            if let Inline::Span { text, .. } = inline {
                *text = apply_str(p, text);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hints(keep: &[&str], hyphenate: &[&str]) -> Prepared {
        prepare(&Hints { keep: keep.iter().map(|s| (*s).to_owned()).collect(), hyphenate: hyphenate.iter().map(|s| (*s).to_owned()).collect() })
    }

    #[test]
    fn phrases_are_kept_together_as_whole_words() {
        let h = hints(&["Mr. Smith", "10 kg"], &[]);
        assert_eq!(apply_str(&h, "Then Mr. Smith  left with 10 kg."), "Then Mr.\u{A0}Smith  left with 10\u{A0}kg.");
        assert_eq!(
            apply_str(&h, "MR. SMITH, and Mr. Smithson"),
            "MR.\u{A0}SMITH, and Mr. Smithson",
            "case is ignored; part of a word is not matched"
        );
        assert_eq!(apply_str(&h, "110 kg"), "110 kg");
        assert_eq!(apply_str(&h, "Mr. Smi\u{AD}th"), "Mr.\u{A0}Smith", "a kept word loses its soft hyphens");
    }

    #[test]
    fn words_break_only_where_the_hint_says() {
        let h = hints(&[], &["man-u-script", "Wordsmith"]);
        assert_eq!(apply_str(&h, "The Manuscript's manuscript."), "The Manuscript's man\u{AD}u\u{AD}script.");
        assert_eq!(apply_str(&h, "MANUSCRIPT"), "MAN\u{AD}U\u{AD}SCRIPT");
        assert_eq!(apply_str(&h, "manu\u{AD}script"), "man\u{AD}u\u{AD}script", "the text's own soft hyphens give way");
        assert_eq!(apply_str(&h, "Word\u{AD}smith and wordsmiths"), "Wordsmith and wordsmiths", "a word without a mark is never hyphenated");
        // A kept phrase wins over a word's breaks.
        let both = hints(&["manuscript page"], &["man-u-script"]);
        assert_eq!(apply_str(&both, "a manuscript page"), "a manuscript\u{A0}page");
    }

    #[test]
    fn unusable_hints_are_left_out() {
        for bad in ["-ab", "ab-", "a--b", "two words", "a-b c"] {
            assert!(breaks_of(bad).is_none(), "{bad}");
        }
        let h = hints(&["", "  ", "..."], &["", "-x"]);
        assert!(h.breaks.is_empty());
        assert_eq!(h.keep.len(), 1, "punctuation alone is a phrase of one token, harmless");
        assert!(hints(&[], &[]).is_empty());
        assert_eq!(apply_str(&hints(&[], &[]), "a\u{AD}b"), "a\u{AD}b", "no hints: the text as it is");
    }

    #[test]
    fn notes_and_every_kind_of_block_are_covered() {
        use wordsmith_manuscript_model::{Block, Note};
        let mut t = Text {
            blocks: vec![Block::Para(vec![Inline::plain("Mr. Smith")]), Block::Quote(vec![vec![Inline::plain("Mr. Smith")]])],
            notes: vec![Note { label: "1".into(), paras: vec![vec![Inline::plain("Mr. Smith")]] }],
        };
        apply(&hints(&["Mr. Smith"], &[]), &mut t);
        assert_eq!(t.inline_runs().iter().filter(|r| r == &&&vec![Inline::plain("Mr.\u{A0}Smith")]).count(), 3);
    }
}
