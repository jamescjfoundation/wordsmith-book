// SPDX-License-Identifier: MIT OR Apache-2.0
//! EPUB import (plan R3.4, IMP-9): an EPUB Book exported imports back word for word, with the
//! book's structure (parts, chapters, front and back matter) from the EPUB's own package; the
//! ebook's own contents and the number lines Book makes again are left out and reported; an EPUB
//! without structure marks imports through detection as any document does.

mod common;

use common::*;
use wordsmith_book_check::{build, verify};
use wordsmith_book_import::Format;
use wordsmith_book_model::Edition;
use wordsmith_manuscript_format::book::{Book, Section, from_project};

/// The body's parts and chapters: kind, title and how many sections each holds.
fn shape(sections: &[Section]) -> Vec<(String, usize)> {
    sections.iter().map(|s| (format!("{:?} {}", s.role, s.title.trim()), s.children.len())).collect()
}

/// The titles of front or back matter pages, the title page apart.
fn matter(sections: &[Section]) -> Vec<String> {
    sections.iter().filter(|s| s.kind != wordsmith_manuscript_format::book::FrontKind::TitlePage).map(|s| s.title.trim().to_owned()).collect()
}

/// Whether `want` appears in `all` in order (other items may come between: the pages Book made
/// for the EPUB, such as a half title, come back too).
fn in_order(all: &[String], want: &[String]) -> bool {
    let mut it = all.iter();
    want.iter().all(|w| it.any(|a| a == w))
}

/// The words of the body: every part and chapter, in order.
fn body_words(b: &Book) -> Vec<String> {
    let only =
        Book { version: b.version, metadata: b.metadata.clone(), front: Vec::new(), body: b.body.clone(), back: Vec::new(), assets: Vec::new() };
    verify::book_words(&only, false)
}

#[test]
fn an_exported_epub_imports_back_word_for_word_with_its_structure() {
    for (name, book) in [("Gulliver's Travels", gulliver()), ("Waverley", waverley())] {
        let (_, e) = build::ebook(&project(), &Default::default(), &book, &Edition::ebook(), &|_| None).unwrap();
        // The import's own check: every word of the EPUB's text is in the import (it refuses
        // otherwise), the contents and number lines apart.
        let i = wordsmith_book_import::import(Format::Epub, &e.bytes, &Default::default(), name).unwrap();
        let back = from_project(&i.project);
        assert_eq!(shape(&back.body), shape(&book.body), "{name}: the same parts and chapters");
        assert!(in_order(&matter(&back.front), &matter(&book.front)), "{name}: {:?} keeps {:?}", matter(&back.front), matter(&book.front));
        assert!(in_order(&matter(&back.back), &matter(&book.back)), "{name}: {:?} keeps {:?}", matter(&back.back), matter(&book.back));
        if let Err(d) = verify::compare(&body_words(&book), &body_words(&back)) {
            panic!("{name}: the body differs: {d}");
        }
        assert_eq!(i.report.dropped.get("the ebook's own contents"), Some(&1), "{name}");
        // The notes come back too.
        let notes = |b: &Book| verify::book_words(b, true).len() - verify::book_words(b, false).len();
        assert_eq!(notes(&back) > 0, notes(&book) > 0, "{name}: notes");
    }
}

#[test]
fn an_epub_without_structure_marks_imports_through_detection() {
    // A small EPUB written by hand: two chapter documents with no `epub:type`, and no navigation
    // in the reading order.
    let container = r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
    let opf = r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="id">x</dc:identifier><dc:title>Two Tales</dc:title><dc:language>en</dc:language></metadata><manifest><item id="a" href="a.xhtml" media-type="application/xhtml+xml"/><item id="b" href="b.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/></manifest><spine><itemref idref="a"/><itemref idref="b"/></spine></package>"#;
    let page = |h: &str, p: &str| {
        format!(
            r#"<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>{h}</title></head><body><h1>{h}</h1><p>{p}</p></body></html>"#
        )
    };
    let nav = r#"<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="a.xhtml">Chapter 1</a></li></ol></nav></body></html>"#;
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let stored = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    use std::io::Write as _;
    for (name, text) in [
        ("mimetype", "application/epub+zip".to_owned()),
        ("META-INF/container.xml", container.to_owned()),
        ("OEBPS/content.opf", opf.to_owned()),
        ("OEBPS/a.xhtml", page("Chapter 1", "The first tale begins in the rain.")),
        ("OEBPS/b.xhtml", page("Chapter 2", "The second tale ends in the sun.")),
        ("OEBPS/nav.xhtml", nav.to_owned()),
    ] {
        z.start_file(name, stored).unwrap();
        z.write_all(text.as_bytes()).unwrap();
    }
    let bytes = z.finish().unwrap().into_inner();
    let i = wordsmith_book_import::import(Format::Epub, &bytes, &Default::default(), "Two Tales").unwrap();
    let b = from_project(&i.project);
    let chapters: Vec<&str> = b.body.iter().map(|s| s.title.as_str()).collect();
    assert_eq!(chapters, ["Chapter 1", "Chapter 2"]);
    assert!(verify::book_words(&b, false).join(" ").contains("The second tale ends in the sun."));
    assert!(!i.report.dropped.contains_key("the ebook's own contents"), "the navigation was not in the reading order");
}
