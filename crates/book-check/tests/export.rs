// SPDX-License-Identifier: MIT OR Apache-2.0
//! Export (plan M8.3, EXP-1, EXP-2): every export twice gives the same bytes; a failing word
//! check writes nothing.

mod common;

use std::path::{Path, PathBuf};

use common::*;
use wordsmith_book_check::export::{self, Error};
use wordsmith_book_format::source::{self, Opened};
use wordsmith_book_model::{Edition, Source};
use wordsmith_manuscript_model::{Block, Inline, Role};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn gulliver_source() -> Opened {
    source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap()
}

fn editions() -> wordsmith_book_model::BookProject {
    let mut p = project();
    let mut pb = Edition::paperback();
    pb.trim = "6x9".into();
    p.editions = vec![pb, Edition::ebook()];
    p
}

#[test]
fn every_export_twice_gives_the_same_bytes() {
    let src = gulliver_source();
    let p = editions();
    let ids: Vec<String> = p.editions.iter().map(|e| e.id.clone()).collect();
    let a = export::build_all(&p, &Default::default(), &src, &ids, &|_| None, None).unwrap();
    let b = export::build_all(&p, &Default::default(), &src, &ids, &|_| None, None).unwrap();
    assert_eq!(a.len(), 2);
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.name, y.name);
        assert!(x.bytes == y.bytes, "{}: two exports differ", x.name);
    }
    assert!(a[0].name.ends_with(" - Paperback.pdf") && a[1].name.ends_with(" - Ebook.epub"), "{} {}", a[0].name, a[1].name);
    let dir = tmp("export-twice");
    let paths = export::write(&dir, &a.iter().map(|x| (x.name.clone(), x.bytes.clone())).collect::<Vec<_>>()).unwrap();
    for (path, x) in paths.iter().zip(&a) {
        assert_eq!(std::fs::read(path).unwrap(), x.bytes);
    }
    assert!(export::write(&dir, &[("../x.pdf".into(), vec![1])]).is_err());
}

fn first_chapter(sections: &mut [wordsmith_manuscript_format::book::Section]) -> Option<&mut wordsmith_manuscript_format::book::Section> {
    for s in sections {
        if s.role == Role::Chapter && !s.content.is_empty() {
            return Some(s);
        }
        if let Some(c) = first_chapter(&mut s.children) {
            return Some(c);
        }
    }
    None
}

#[test]
fn a_failing_word_check_writes_nothing() {
    // Words in a script the book's fonts do not have cannot be printed as written.
    let mut src = gulliver_source();
    let ch = first_chapter(&mut src.book.body).unwrap();
    if let Some(Block::Para(c)) = ch.content.get_mut(2)
        && let Some(Inline::Span { text, .. }) = c.first_mut()
    {
        *text = format!("東京 {text}");
    }
    let p = editions();
    let err = export::build_all(&p, &Default::default(), &src, &["paperback".into(), "ebook".into()], &|_| None, None).unwrap_err();
    let Error::Blocked(report) = &err else { panic!("{err}") };
    assert!(report.findings.iter().any(|f| f.rule == "words"), "{:#?}", report.findings);
    assert!(err.to_string().starts_with("nothing was exported"), "{err}");
}
