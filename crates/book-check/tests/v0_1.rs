// SPDX-License-Identifier: MIT OR Apache-2.0
//! The test of v0.1 (plan section 10, M9.4), end to end: two public-domain novels, Waverley
//! (about 210,000 words) from its DOCX and Gulliver's Travels (parts, notes and verse) from its DOCX
//! and from the Manuscript project fixture, go to a 6 × 9 in paperback PDF and an EPUB in each of
//! the six designs of 0.1 (later designs have their golden files in the print and EPUB golden
//! tests). Every word of both outputs matches the source; the PDFs pass preflight for
//! `pod-paperback` and match their golden files (`tests/golden/v0.1/`, docs/golden.md); the
//! EPUBs pass Book's structural checks (and the standard EPUB validator where it is set up,
//! `BOOK_EPUB_VALIDATOR`); building twice gives the same bytes. `BOOK_BLESS=1` writes the golden
//! files again.

mod common;

/// The designs 0.1 shipped: the test of v0.1 stays the test of what 0.1 made.
const V0_1_DESIGNS: [&str; 6] = ["classic", "literary", "romance", "thriller", "fantasy", "non-fiction"];

use std::path::{Path, PathBuf};

use common::*;
use wordsmith_book_check::{build, epub, preflight, validator, verify};
use wordsmith_book_design::builtin;
use wordsmith_book_format::source::{self, Opened};
use wordsmith_book_model::{Edition, Severity, Source};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A DOCX imported into a source of its own, opened as Book opens it.
fn from_docx(name: &str, title: &str, author: &str) -> Opened {
    let dir = tmp(&format!("v0.1-{title}"));
    let bytes = std::fs::read(novels().join(name)).unwrap();
    let mut i = wordsmith_book_import::import(wordsmith_book_import::Format::Docx, &bytes, &Default::default(), title).unwrap();
    assert!(i.report.passed(), "{name}: the import's word check failed");
    i.project.settings.book.author = author.into();
    source::write_owned(&dir, "source.wsm", &i.project).unwrap();
    source::open(&dir, &Source::Owned("source.wsm".into())).unwrap()
}

fn paperback() -> Edition {
    let mut e = Edition::paperback();
    e.trim = "6x9".into();
    e.profile = "pod-paperback".into();
    e
}

#[test]
fn the_test_of_v0_1() {
    let bless = std::env::var_os("BOOK_BLESS").is_some();
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/v0.1");
    let validator = std::env::var("BOOK_EPUB_VALIDATOR").ok().map(|program| validator::Settings {
        program,
        java: std::env::var("BOOK_JAVA").unwrap_or_default(),
        seconds: 300,
        access: String::new(),
        pdf: String::new(),
    });
    let sources = [
        ("waverley-docx", from_docx("waverley.docx", "Waverley", "Walter Scott")),
        ("gulliver-docx", from_docx("gullivers-travels.docx", "Gulliver's Travels", "Jonathan Swift")),
        ("gulliver-manuscript", source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap()),
    ];
    // One novel is long enough, and between them they have parts, notes and verse.
    assert!(verify::book_words(&sources[0].1.book, true).len() >= 150_000);
    for (what, found) in [
        ("parts", sources.iter().any(|(_, s)| s.book.body.iter().any(|x| x.role == wordsmith_manuscript_model::Role::Part))),
        ("notes", sources.iter().any(|(_, s)| format!("{:?}", s.book).contains("NoteRef"))),
        // Verse: a verse block, or (from a word processor) a stanza kept with its line breaks.
        (
            "verse",
            sources.iter().any(|(_, s)| {
                let d = format!("{:?}", s.book);
                d.contains("Verse") || d.contains("\\n")
            }),
        ),
    ] {
        assert!(found, "the test books have no {what}");
    }
    let mut failures = Vec::new();
    for (name, src) in &sources {
        for d in builtin::all().into_iter().filter(|d| V0_1_DESIGNS.contains(&d.id.as_str())) {
            let label = format!("{name}-{}", d.id);
            let mut p = project();
            p.design.id = d.id.clone();
            p.editions = vec![paperback(), Edition::ebook()];
            let book = wordsmith_book_format::fixes::book_for_build(&p.fixes, src);
            // The paperback: preflight (which checks every letter and word), golden, twice.
            let (r, pdf) = preflight::preflight(&p, &Default::default(), src, &p.editions[0], &|_| None);
            let pdf = pdf.unwrap_or_else(|| panic!("{label}: no PDF"));
            assert!(!r.blocked(), "{label}: {:#?}", r.of(Severity::Blocks));
            let pr = build::print(&p, &Default::default(), &book, &p.editions[0], &|_| None).unwrap();
            let again = pr.typeset.pdf().unwrap();
            assert!(again == pdf, "{label}: two builds of the PDF differ");
            let got = summary(&pr, &pdf);
            let path = golden.join(format!("{label}.txt"));
            if bless {
                std::fs::create_dir_all(&golden).unwrap();
                std::fs::write(&path, &got).unwrap();
            } else if std::fs::read_to_string(&path).ok().as_deref() != Some(got.as_str()) {
                failures.push(format!("{label}: the PDF does not match its golden file"));
            }
            // The ebook: structure, words, validator, twice.
            let (r, epub_bytes) = preflight::preflight(&p, &Default::default(), src, &p.editions[1], &|_| None);
            let epub_bytes = epub_bytes.unwrap_or_else(|| panic!("{label}: no EPUB"));
            assert!(!r.blocked(), "{label} ebook: {:#?}", r.of(Severity::Blocks));
            assert!(epub::check(&epub_bytes).is_empty(), "{label}: {:#?}", epub::check(&epub_bytes));
            let words = epub::words(&epub_bytes, true).unwrap();
            verify::compare(&verify::book_words(&book, true), &words).unwrap_or_else(|e| panic!("{label}: {e}"));
            let (_, second) = build::ebook(&p, &Default::default(), &book, &p.editions[1], &|_| None).unwrap();
            assert!(second.bytes == epub_bytes, "{label}: two builds of the EPUB differ");
            if let Some(v) = &validator {
                let found = validator::check(v, &epub_bytes);
                assert!(found.iter().all(|f| f.severity == Severity::Note), "{label}: {found:#?}");
            }
        }
    }
    assert!(failures.is_empty(), "{}\nRun with BOOK_BLESS=1 after looking at the PDFs.", failures.join("\n"));
}
