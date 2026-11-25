// SPDX-License-Identifier: MIT OR Apache-2.0
//! Notes (plan R2.4, TXT-3, EBK-7): print footnotes on the page that calls them, a long one split
//! across pages; chapter-end and book-end notes per edition; the ebook's pop-up footnotes and
//! endnotes linked both ways; every word checked with the notes, in each style.

mod common;

use common::*;
use wordsmith_book_check::{build, epub, preflight, verify};
use wordsmith_book_format::source;
use wordsmith_book_model::{Edition, NoteStyle, Severity, Source};
use wordsmith_manuscript_model::{Inline, Role};

fn edition(style: NoteStyle, print: bool) -> Edition {
    let mut e = if print { Edition::paperback() } else { Edition::ebook() };
    e.notes = style;
    e
}

/// For each footnote part: the pages of the paragraph that calls it, and the pages its lines are
/// on.
fn footnote_pages(t: &wordsmith_book_typeset::Typeset) -> Vec<(Vec<usize>, Vec<usize>)> {
    let doc = t.document();
    let layout = t.layout();
    let pieces = jcj_layout::pages::body_pieces(layout);
    let mut out = Vec::new();
    for (path, p) in jcj_doc::walk::body_paragraphs(doc) {
        for (_, o) in p.objects_with_offsets() {
            if let jcj_doc::para::InlineObject::NoteRef { id, .. } = o {
                let block = path.0[0] as usize;
                let ref_page: Vec<usize> = pieces.iter().enumerate().filter(|(_, pp)| pp.iter().any(|x| x.block == block)).map(|(i, _)| i).collect();
                let on: Vec<usize> = layout
                    .pages
                    .iter()
                    .enumerate()
                    .filter(|(_, pg)| {
                        pg.items.iter().any(|it| matches!(it, jcj_layout::Placed::Lines { story: jcj_doc::StoryRef::Part(x), .. } if x == id))
                    })
                    .map(|(i, _)| i)
                    .collect();
                out.push((ref_page, on));
            }
        }
    }
    out
}

#[test]
fn footnotes_print_on_the_page_that_calls_them() {
    let pr = build::print(&project(), &Default::default(), &gulliver(), &edition(NoteStyle::Popup, true), &|_| None).unwrap();
    let notes = footnote_pages(&pr.typeset);
    let total: usize = gulliver_notes();
    assert_eq!(notes.len(), total, "every note is a footnote");
    for (calling, on) in &notes {
        assert!(on.first().is_some_and(|p| calling.contains(p)), "a footnote starts on a page of the paragraph that calls it: {notes:?}");
    }
    // Nothing is printed as a chapter-end list.
    let text: String = pr.typeset.printed_paragraphs().iter().map(|p| p.0.clone()).collect::<Vec<_>>().join("\n");
    assert!(!text.lines().any(|l| l == "Notes"));
    assert!(preflight::print_words(&pr, &pr.typeset.pdf().unwrap(), &gulliver()).is_empty());
}

fn gulliver_notes() -> usize {
    fn count(s: &wordsmith_manuscript_format::book::Section) -> usize {
        s.notes.len() + s.children.iter().map(count).sum::<usize>()
    }
    let b = gulliver();
    b.front.iter().chain(&b.body).chain(&b.back).map(count).sum()
}

#[test]
fn a_long_footnote_continues_on_the_next_page() {
    let mut b = gulliver();
    fn first(s: &mut [wordsmith_manuscript_format::book::Section]) -> Option<&mut wordsmith_manuscript_format::book::Section> {
        for x in s {
            if x.role == Role::Chapter && !x.notes.is_empty() {
                return Some(x);
            }
            if let Some(c) = first(&mut x.children) {
                return Some(c);
            }
        }
        None
    }
    let ch = first(&mut b.body).expect("a chapter with notes");
    let long = "This note goes on at length about the matter in hand, and then some more. ".repeat(60);
    ch.notes[0].paras = vec![vec![Inline::Span { text: long, marks: Default::default() }]];
    let pr = build::print(&project(), &Default::default(), &b, &edition(NoteStyle::Popup, true), &|_| None).unwrap();
    let split = footnote_pages(&pr.typeset).into_iter().find(|(_, on)| on.len() > 1).expect("the long note is split");
    assert_eq!(split.1[1], split.1[0] + 1, "it continues on the next page: {split:?}");
    assert!(preflight::print_words(&pr, &pr.typeset.pdf().unwrap(), &b).is_empty(), "every letter of the split note prints");
}

#[test]
fn each_note_style_passes_preflight_in_print_and_ebook() {
    let src = source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap();
    for style in [NoteStyle::Popup, NoteStyle::ChapterEnd, NoteStyle::BookEnd] {
        for print in [true, false] {
            let e = edition(style, print);
            let (r, bytes) = preflight::preflight(&project(), &Default::default(), &src, &e, &|_| None);
            assert!(!r.blocked(), "{style:?} {}: {:#?}", e.id, r.of(Severity::Blocks));
            let bytes = bytes.unwrap();
            if !print {
                let words = epub::words(&bytes, true).unwrap();
                let want = if style == NoteStyle::BookEnd { verify::book_words_notes_last(&src.book) } else { verify::book_words(&src.book, true) };
                verify::compare(&want, &words).unwrap();
            }
        }
    }
    // Book-end notes print under a Notes section at the book's end.
    let pr = build::print(&project(), &Default::default(), &gulliver(), &edition(NoteStyle::BookEnd, true), &|_| None).unwrap();
    let paras = pr.typeset.printed_paragraphs();
    let notes_at = paras.iter().position(|p| p.0 == "Notes").expect("a Notes heading");
    assert!(paras[notes_at..].len() > gulliver_notes());
}

#[test]
fn ebook_notes_link_both_ways_in_each_style() {
    for style in [NoteStyle::Popup, NoteStyle::ChapterEnd, NoteStyle::BookEnd] {
        let (_, e) = build::ebook(&project(), &Default::default(), &gulliver(), &edition(style, false), &|_| None).unwrap();
        assert!(epub::check(&e.bytes).is_empty(), "{style:?}: {:#?}", epub::check(&e.bytes));
        let files = epub::unzip(&e.bytes).unwrap();
        let all: String = files.iter().filter(|f| f.0.ends_with(".xhtml")).map(|f| String::from_utf8_lossy(&f.2).into_owned()).collect();
        let refs = all.matches("epub:type=\"noteref\"").count();
        let backs = all.matches("role=\"doc-backlink\"").count();
        assert_eq!(refs, gulliver_notes(), "{style:?}");
        assert_eq!(backs, refs, "{style:?}");
        match style {
            NoteStyle::Popup => assert_eq!(all.matches("epub:type=\"footnote\"").count(), refs),
            NoteStyle::ChapterEnd => assert_eq!(all.matches("epub:type=\"endnote\"").count(), refs),
            NoteStyle::BookEnd => {
                assert!(files.iter().any(|f| f.0 == "EPUB/text/notes.xhtml"));
                assert!(all.contains("href=\"notes.xhtml#"));
            }
        }
    }
}
