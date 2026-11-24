// SPDX-License-Identifier: MIT OR Apache-2.0
//! Structure detection for imported documents (IMP-4): parts, chapters, scene breaks and front
//! and back matter from heading styles, page breaks, "Chapter N" and numbered lines, capital
//! lines after page breaks and symbol lines. Every rule has a name and can be switched off in the
//! import review; the choices are kept in `book.toml` for re-imports.
//!
// core-candidate: open-source-core#22 (C8). Nothing here knows about books beyond the words it
// looks for: the caller describes each paragraph and gets each one's part in the structure.

use wordsmith_book_model::ImportRules;

/// What detection needs to know about one paragraph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Para {
    /// Its text.
    pub text: String,
    /// Its heading level from its style (1 is the highest), if it has a heading style.
    pub level: Option<u8>,
    /// Every run of it is bold.
    pub bold: bool,
    /// It is centered.
    pub centered: bool,
    /// A page break comes right before it.
    pub after_page_break: bool,
}

/// The kind of a section a heading starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A part.
    Part,
    /// A chapter.
    Chapter,
    /// Front matter.
    Front,
    /// Back matter.
    Back,
}

/// What one paragraph is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Body text.
    Body,
    /// A heading that starts a section, with the rule that found it.
    Heading(Kind, &'static str),
    /// A scene break, with the rule that found it.
    Break(&'static str),
    /// Nothing: an empty paragraph or a page break (listed in the report as dropped).
    Skip,
}

/// The rules, by name, with what each does (for the review).
pub const RULES: [(&str, &str); 10] = [
    ("heading-styles", "Paragraphs with heading styles start parts and chapters"),
    ("part-lines", "Lines like “Part Two” or “PART I. ...” start parts"),
    ("chapter-lines", "Lines like “Chapter 12” or “CHAPTER XII.” start chapters"),
    ("number-lines", "A line that is only a number (“12”, “XII”) starts a chapter"),
    ("caps-lines", "A short line in capitals after a page break starts a chapter"),
    ("symbol-breaks", "A line of symbols (* * *, #, ~) is a scene break"),
    ("empty-paragraph-breaks", "Two or more empty paragraphs are a scene break, when there are no symbol lines"),
    ("matter-titles", "Headings like “Dedication” or “About the Author” are front or back matter"),
    ("underline-italic", "Underlined text is shown in italic"),
    ("underscores-italic", "In plain text, _underscored_ words are italic"),
];

/// The page break character inside a paragraph's text.
pub const PAGE_BREAK: char = '\u{c}';

const NUMBER_WORDS: [&str; 30] = [
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
    "thirty",
    "forty",
    "fifty",
    "sixty",
    "seventy",
    "eighty",
    "ninety",
    "hundred",
    "first",
    "last",
];

const FRONT: [&str; 12] = [
    "title page",
    "half title",
    "copyright",
    "dedication",
    "epigraph",
    "contents",
    "table of contents",
    "foreword",
    "preface",
    "introduction",
    "prologue",
    "a note on the text",
];

const BACK: [&str; 13] = [
    "about the author",
    "acknowledgements",
    "acknowledgments",
    "afterword",
    "epilogue",
    "glossary",
    "notes",
    "footnotes",
    "endnotes",
    "author's note",
    "author’s note",
    "also by",
    "discussion questions",
];

fn is_on(rules: &ImportRules, name: &str) -> bool {
    !rules.off.iter().any(|r| r == name)
}

/// Whether `s` is a number: figures, roman numerals, or number words ("Twenty-One").
pub fn is_number(s: &str) -> bool {
    let s = s.trim().trim_end_matches(['.', ':', ')']);
    if s.is_empty() {
        return false;
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return s.len() <= 4;
    }
    if s.chars().all(|c| "IVXLCivxlc".contains(c)) {
        return s.len() <= 8;
    }
    let lower = s.to_lowercase();
    lower.split(['-', ' ']).filter(|w| !w.is_empty()).all(|w| NUMBER_WORDS.iter().any(|n| w.starts_with(n.trim_end_matches('y')) || w == *n))
}

fn starts_with_word(text: &str, words: &[String]) -> Option<String> {
    let t = text.trim();
    let first = t.split_whitespace().next()?;
    let rest = t[first.len()..].trim_start();
    let num = rest.split_whitespace().next().unwrap_or("");
    words.iter().find(|w| first.eq_ignore_ascii_case(w) && is_number(num.trim_end_matches(['.', ':']))).cloned()
}

fn symbol_line(text: &str, extra: &[String]) -> bool {
    let t = text.trim();
    if t.is_empty() {
        return false;
    }
    if extra.iter().any(|m| m == t) {
        return true;
    }
    t.chars().all(|c| c.is_whitespace() || "*#~•⁂·-–—=_+✻✦❦❧§".contains(c)) && t.chars().filter(|c| !c.is_whitespace()).count() <= 12
}

fn matter_kind(text: &str) -> Option<Kind> {
    let t = text.trim().trim_end_matches(['.', ':']).to_lowercase();
    if FRONT.iter().any(|f| t == *f) {
        return Some(Kind::Front);
    }
    if BACK.iter().any(|b| t == *b || (b == &"also by" && t.starts_with("also by"))) {
        return Some(Kind::Back);
    }
    None
}

/// Whether a paragraph is empty: no text, or only a page break.
pub fn is_empty(p: &Para) -> bool {
    p.text.chars().all(|c| c.is_whitespace() || c == PAGE_BREAK)
}

/// Mark each paragraph.
pub fn detect(paras: &[Para], rules: &ImportRules) -> Vec<Mark> {
    let mut part_words = vec!["Part".to_owned(), "Book".to_owned(), "Volume".to_owned()];
    let mut chapter_words = vec!["Chapter".to_owned()];
    chapter_words.extend(rules.chapter_words.iter().cloned());
    part_words.retain(|w| !chapter_words.iter().any(|c| c.eq_ignore_ascii_case(w)));
    let has_symbols = is_on(rules, "symbol-breaks") && paras.iter().any(|p| symbol_line(&p.text, &rules.break_marks));
    // The heading level chapters use: the deepest heading level in use, when two are.
    let levels: std::collections::BTreeSet<u8> = paras.iter().filter_map(|p| p.level).collect();
    let chapter_level = if levels.len() >= 2 { levels.iter().nth(1).copied() } else { levels.first().copied() };
    let mut marks = Vec::with_capacity(paras.len());
    let mut empties = 0usize;
    for p in paras {
        let text = p.text.trim();
        if is_empty(p) {
            empties += 1;
            marks.push(Mark::Skip);
            continue;
        }
        // A run of empty paragraphs between body paragraphs is a scene break (when the source has
        // no symbol lines).
        if empties >= 2
            && !has_symbols
            && is_on(rules, "empty-paragraph-breaks")
            && matches!(marks.iter().rev().find(|m| **m != Mark::Skip), Some(Mark::Body))
            && let Some(last) = marks.last_mut()
        {
            *last = Mark::Break("empty-paragraph-breaks");
        }
        empties = 0;
        let short = text.chars().count() <= 80;
        let mark = if has_symbols && symbol_line(text, &rules.break_marks) {
            Mark::Break("symbol-breaks")
        } else if is_on(rules, "matter-titles")
            && short
            && (p.level.is_some() || p.bold || p.centered || p.after_page_break)
            && let Some(k) = matter_kind(text)
        {
            Mark::Heading(k, "matter-titles")
        } else if is_on(rules, "part-lines") && short && starts_with_word(text, &part_words).is_some() {
            Mark::Heading(Kind::Part, "part-lines")
        } else if is_on(rules, "chapter-lines") && short && starts_with_word(text, &chapter_words).is_some() {
            Mark::Heading(Kind::Chapter, "chapter-lines")
        } else if is_on(rules, "heading-styles")
            && let Some(level) = p.level
        {
            let kind = if levels.len() >= 2 && Some(level) < chapter_level { Kind::Part } else { Kind::Chapter };
            Mark::Heading(kind, "heading-styles")
        } else if is_on(rules, "number-lines") && short && is_number(text) && (p.bold || p.centered || p.after_page_break || p.level.is_some()) {
            Mark::Heading(Kind::Chapter, "number-lines")
        } else if is_on(rules, "caps-lines")
            && p.after_page_break
            && short
            && text.chars().any(char::is_alphabetic)
            && text.chars().filter(|c| c.is_alphabetic()).all(char::is_uppercase)
        {
            Mark::Heading(Kind::Chapter, "caps-lines")
        } else {
            Mark::Body
        };
        marks.push(mark);
    }
    // Headings before the first part or chapter are front matter. When the book has parts,
    // headings before the first part are too, unless their words say they are chapters.
    let first_part = marks.iter().position(|m| matches!(m, Mark::Heading(Kind::Part, _)));
    let first_section = first_part.or_else(|| marks.iter().position(|m| matches!(m, Mark::Heading(Kind::Chapter, _))));
    if let Some(first) = first_section {
        for m in marks.iter_mut().take(first) {
            if let Mark::Heading(kind, rule) = m
                && !(*kind == Kind::Chapter && matches!(*rule, "chapter-lines" | "number-lines"))
            {
                *m = Mark::Heading(Kind::Front, rule);
            }
        }
    }
    marks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(text: &str) -> Para {
        Para { text: text.into(), ..Para::default() }
    }

    fn bold(text: &str) -> Para {
        Para { text: text.into(), bold: true, ..Para::default() }
    }

    #[test]
    fn numbers_in_figures_roman_and_words() {
        for n in ["12", "XII", "xii.", "Twelve", "Twenty-One", "One Hundred"] {
            assert!(is_number(n), "{n}");
        }
        for n in ["Twelfth Night", "Lighthouse", "", "12345"] {
            assert!(!is_number(n), "{n}");
        }
    }

    #[test]
    fn chapter_lines_parts_breaks_and_matter() {
        let paras = vec![
            bold("Dedication"),
            p("For you."),
            bold("Part One"),
            bold("Chapter 1"),
            p("It began."),
            p("* * *"),
            p("It went on."),
            bold("CHAPTER II."),
            p("It ended."),
            bold("About the Author"),
            p("A writer."),
        ];
        let m = detect(&paras, &ImportRules::default());
        assert_eq!(m[0], Mark::Heading(Kind::Front, "matter-titles"));
        assert_eq!(m[2], Mark::Heading(Kind::Part, "part-lines"));
        assert_eq!(m[3], Mark::Heading(Kind::Chapter, "chapter-lines"));
        assert_eq!(m[5], Mark::Break("symbol-breaks"));
        assert_eq!(m[7], Mark::Heading(Kind::Chapter, "chapter-lines"));
        assert_eq!(m[9], Mark::Heading(Kind::Back, "matter-titles"));
        let off = ImportRules { off: vec!["chapter-lines".into()], ..ImportRules::default() };
        assert_eq!(detect(&paras, &off)[3], Mark::Body);
    }

    #[test]
    fn heading_levels_and_empty_paragraph_breaks() {
        let h = |t: &str, l: u8| Para { text: t.into(), level: Some(l), ..Para::default() };
        let paras = vec![h("Volume I.", 1), h("The Start", 2), p("One."), p(""), p(""), p("Two."), h("The End", 2), p("Three.")];
        let m = detect(&paras, &ImportRules::default());
        assert_eq!(m[0], Mark::Heading(Kind::Part, "part-lines"));
        assert_eq!(m[1], Mark::Heading(Kind::Chapter, "heading-styles"));
        assert_eq!(m[4], Mark::Break("empty-paragraph-breaks"));
        assert_eq!(m[3], Mark::Skip);
    }
}
