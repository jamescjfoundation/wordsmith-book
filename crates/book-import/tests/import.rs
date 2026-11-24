// SPDX-License-Identifier: MIT OR Apache-2.0
//! Imports (plan M2.2): every DOCX, ODT and RTF fixture imported with its structure and every
//! word checked; the online word processor's download found with the default rules; a source
//! whose words would change refused.

use std::path::{Path, PathBuf};

use wordsmith_book_import::{Error, Format, Imported, Report, check, import, read};
use wordsmith_book_model::ImportRules;
use wordsmith_manuscript_model::{Block, Role};

fn novels() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels")
}

fn imported(name: &str, format: Format) -> Imported {
    let bytes = std::fs::read(novels().join(name)).unwrap();
    import(format, &bytes, &ImportRules::default(), name).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn made(r: &Report, what: &str) -> usize {
    r.made.get(what).copied().unwrap_or(0)
}

#[test]
fn every_word_processor_fixture_imports_with_its_structure_and_every_word() {
    for (name, format, parts, chapters, front) in [
        ("waverley.docx", Format::Docx, 2, 72, 0),
        ("waverley.odt", Format::Odt, 2, 72, 0),
        ("waverley.rtf", Format::Rtf, 2, 72, 0),
        ("gullivers-travels.docx", Format::Docx, 4, 39, 2),
        ("gullivers-travels.odt", Format::Odt, 4, 39, 2),
        ("gullivers-travels.rtf", Format::Rtf, 4, 39, 2),
    ] {
        let i = imported(name, format);
        assert!(i.report.passed(), "{name}: {:?}", i.report.words);
        assert_eq!(
            (made(&i.report, "parts"), made(&i.report, "chapters"), made(&i.report, "front matter")),
            (parts, chapters, front),
            "{name}: {:?}",
            i.report.made
        );
        // Through the book view, as Book will read the owned source.
        let book = wordsmith_manuscript_format::book::from_project(&i.project);
        assert_eq!(book.body.len(), parts, "{name}");
        assert!(book.body.iter().all(|p| p.role == Role::Part));
        let with_notes = book.body.iter().flat_map(|p| &p.children).filter(|c| !c.notes.is_empty()).count();
        assert!(with_notes > 0, "{name}: notes are kept as notes");
        assert!(i.report.kept.contains_key("italic"), "{name}");
    }
}

#[test]
fn the_online_word_processors_download_is_found_with_the_default_rules() {
    let i = imported("gullivers-travels-online.docx", Format::Docx);
    let r = &i.report;
    assert!(r.passed(), "{:?}", r.words);
    assert_eq!(made(r, "parts"), 4, "{:?}", r.made);
    assert_eq!(made(r, "chapters"), 39);
    assert!(made(r, "scene breaks") > 20, "{:?}", r.made);
    // Fonts and sizes on every run, empty paragraphs and page breaks: dropped and listed.
    for what in ["font", "size", "empty paragraphs", "page breaks"] {
        assert!(r.dropped.get(what).copied().unwrap_or(0) > 0, "{what} dropped: {:?}", r.dropped);
    }
    // Every uneven mark became the same scene break.
    let book = wordsmith_manuscript_format::book::from_project(&i.project);
    let breaks: usize = book.body.iter().flat_map(|p| &p.children).map(|c| c.content.iter().filter(|b| **b == Block::SceneBreak).count()).sum();
    assert_eq!(breaks, made(r, "scene breaks"));
    // The bold title before the first part is the title page.
    assert_eq!(book.front.len(), 1);
    // With the chapter-line rule off, no chapters are found by their lines.
    let bytes = std::fs::read(novels().join("gullivers-travels-online.docx")).unwrap();
    let off = import(Format::Docx, &bytes, &ImportRules { off: vec!["chapter-lines".into()], ..ImportRules::default() }, "x").unwrap();
    assert_eq!(made(&off.report, "chapters"), 0);
    assert!(off.report.passed());
}

#[test]
fn an_import_whose_words_would_change_is_refused() {
    let bytes = std::fs::read(novels().join("gullivers-travels.docx")).unwrap();
    let doc = read(Format::Docx, &bytes).unwrap();
    let mut i = imported("gullivers-travels.docx", Format::Docx);
    // Change one word of the imported text, as a faulty import would.
    let id = *i.project.texts.keys().nth(5).unwrap();
    let t = i.project.texts.get_mut(&id).unwrap();
    if let Some(Block::Para(c)) = t.body.blocks.iter_mut().find(|b| matches!(b, Block::Para(_))) {
        c.push(wordsmith_manuscript_model::Inline::plain(" extra"));
    }
    let mut report = Report::default();
    let e = check(&doc, &i.project, &i.named, &mut report).unwrap_err();
    assert!(matches!(e, Error::Words(_)));
    assert!(e.to_string().contains("expected"), "{e}");
    assert!(!report.passed());
    // Not a document at all.
    assert!(matches!(import(Format::Docx, b"not a zip", &ImportRules::default(), "x"), Err(Error::Read(_))));
}

/// M2's milestone: every fixture file imports with a passing word check.
#[test]
fn every_fixture_imports_with_a_passing_check() {
    let mut n = 0;
    let mut names: Vec<_> = std::fs::read_dir(novels()).unwrap().flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
    names.sort();
    for path in names {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let Some(format) = Format::from_extension(ext) else { continue };
        let bytes = std::fs::read(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let i = import(format, &bytes, &ImportRules::default(), &name).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(i.report.passed(), "{name}");
        assert!(i.report.made.get("chapters").copied().unwrap_or(0) > 30, "{name}: {:?}", i.report.made);
        n += 1;
    }
    assert_eq!(n, 13, "every text, Markdown, DOCX, ODT and RTF fixture");
}
