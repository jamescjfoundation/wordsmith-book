// SPDX-License-Identifier: MIT OR Apache-2.0
//! Preflight (plan M8.1, PRE-1, PRE-4): for each rule a book or file written to break it, with the
//! finding's severity and link; and the v0.1 test books pass for `pod-paperback`.

mod common;

use std::path::{Path, PathBuf};

use common::*;
use wordsmith_book_check::build;
use wordsmith_book_check::preflight::{self, Finding, Link, Report};
use wordsmith_book_design::builtin;
use wordsmith_book_format::source::{self, Opened};
use wordsmith_book_model::{Edition, Fix, Severity, Source};
use wordsmith_manuscript_format::book::Book;
use wordsmith_manuscript_model::{Block, ImageRole, Role};

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The Manuscript fixture, opened as a linked source (read, never written).
fn gulliver_source() -> Opened {
    source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap()
}

/// Waverley imported from its DOCX into a source of its own.
fn waverley_source() -> Opened {
    let dir = tmp("preflight-waverley");
    let bytes = std::fs::read(novels().join("waverley.docx")).unwrap();
    let mut i = wordsmith_book_import::import(wordsmith_book_import::Format::Docx, &bytes, &Default::default(), "Waverley").unwrap();
    i.project.settings.book.author = "Walter Scott".into();
    source::write_owned(&dir, "source.wsm", &i.project).unwrap();
    source::open(&dir, &Source::Owned("source.wsm".into())).unwrap()
}

fn rules(r: &[Finding], rule: &str) -> Vec<Finding> {
    r.iter().filter(|f| f.rule == rule).cloned().collect()
}

fn show(r: &Report) -> String {
    r.findings.iter().map(|f| format!("{:?} {}: {} ({:?})", f.severity, f.rule, f.message, f.link)).collect::<Vec<_>>().join("\n")
}

fn paperback() -> Edition {
    let mut e = Edition::paperback();
    e.trim = "6x9".into();
    e
}

#[test]
fn the_test_books_pass_preflight_for_the_paperback_and_the_ebook_in_every_design() {
    for (name, src) in [("Gulliver's Travels", gulliver_source()), ("Waverley", waverley_source())] {
        for d in builtin::all() {
            let mut p = project();
            p.design.id = d.id.clone();
            for e in [paperback(), Edition::ebook()] {
                let (r, bytes) = preflight::preflight(&p, &Default::default(), &src, &e, &|_| None);
                assert!(bytes.is_some(), "{name}, {}, {}", d.id, e.id);
                assert!(!r.blocked(), "{name}, {}, {}:\n{}", d.id, e.id, show(&r));
                // Nothing but the book's own missing details and the absent cover is worth a
                // warning.
                for w in r.of(Severity::Warning) {
                    assert!(matches!(w.rule, "metadata" | "cover"), "{name}, {}, {}: {}", d.id, e.id, show(&r));
                }
            }
        }
    }
}

fn gulliver_print(edit: impl FnOnce(&mut wordsmith_book_typeset::Look)) -> build::Print {
    let b = gulliver();
    let mut pr = build::print(&project(), &Default::default(), &b, &paperback(), &|_| None).unwrap();
    edit(&mut pr.look);
    pr.typeset = wordsmith_book_typeset::typeset(&pr.book, &pr.look, "2026-11-24").unwrap();
    pr
}

#[test]
fn narrow_margins_block_export_and_link_to_the_page() {
    let pr = gulliver_print(|l| {
        l.margins = (0.2, 0.1, 0.5, 0.5);
        l.gutter.clear();
    });
    let m = preflight::margins(&pr);
    assert!(!m.is_empty() && m.iter().all(|f| f.severity == Severity::Blocks));
    assert_eq!(m[0].link, Link::Page(1));
    assert!(m[0].message.contains("inside margin") && m[0].message.contains("outside margin"), "{}", m[0].message);
    // The gutter follows the page count: a thin book needs less.
    assert!(preflight::margins(&gulliver_print(|_| {})).is_empty());
}

#[test]
fn small_type_blocks_export() {
    let pr = gulliver_print(|l| l.head_size = 5.5);
    let pdf = pr.typeset.pdf().unwrap();
    let f = rules(&preflight::print_findings(&pr, &pdf, &gulliver(), &paperback()), "type-size");
    assert_eq!(f.len(), 1);
    assert!(f[0].message.contains("5.5 pt"), "{}", f[0].message);
    let pr = gulliver_print(|l| l.body_size = 6.0);
    let pdf = pr.typeset.pdf().unwrap();
    assert!(!rules(&preflight::print_findings(&pr, &pdf, &gulliver(), &paperback()), "type-size").is_empty());
}

/// A PDF written for the test: one page of 300 × 400 pt with a link, a font that is not
/// embedded and a bookmark.
fn hand_made_pdf() -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R /Outlines 6 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 400] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R /Annots [7 0 R] >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Plain >>".to_owned(),
        {
            let c = "BT /F1 12 Tf 50 200 Td (Hi) Tj ET";
            format!("<< /Length {} >>\nstream\n{c}\nendstream", c.len())
        },
        "<< /Type /Outlines /First 8 0 R /Last 8 0 R /Count 1 >>".to_owned(),
        "<< /Type /Annot /Subtype /Link /Rect [0 0 10 10] >>".to_owned(),
        "<< /Title (One) /Parent 6 0 R >>".to_owned(),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());
    out
}

#[test]
fn a_pdf_with_links_bookmarks_and_fonts_not_embedded_is_blocked() {
    let pr = gulliver_print(|_| {});
    let f = preflight::pdf_facts(&pr, &hand_made_pdf());
    let fonts = rules(&f, "fonts");
    assert_eq!(fonts.len(), 1);
    assert!(fonts[0].message.contains("Plain") && fonts[0].severity == Severity::Blocks);
    let ann = rules(&f, "annotations");
    assert_eq!(ann.len(), 2, "{ann:?}");
    assert_eq!(ann[0].link, Link::Page(1));
    assert!(ann.iter().any(|a| a.message.contains("bookmarks")));
    let size = rules(&f, "page-size");
    assert_eq!(size.len(), 1);
    assert_eq!(size[0].link, Link::Page(1));
    // The book's own PDF has none of these.
    let pdf = pr.typeset.pdf().unwrap();
    assert!(preflight::pdf_facts(&pr, &pdf).is_empty());
}

#[test]
fn a_font_that_forbids_embedding_blocks_export() {
    let face = wordsmith_book_design::fonts::regular(wordsmith_book_design::fonts::Role::ClassicSerif).unwrap();
    let mut bytes = face.bytes.to_vec();
    // Set the OS/2 table's embedding bits to "restricted".
    let n = usize::from(u16::from_be_bytes([bytes[4], bytes[5]]));
    let rec = (0..n).map(|i| 12 + 16 * i).find(|&r| &bytes[r..r + 4] == b"OS/2").unwrap();
    let off = u32::from_be_bytes([bytes[rec + 8], bytes[rec + 9], bytes[rec + 10], bytes[rec + 11]]) as usize;
    bytes[off + 8] = 0;
    bytes[off + 9] = 2;
    let f = preflight::font_licenses(&[("Test serif".into(), &bytes), ("Fine".into(), face.bytes)]);
    assert_eq!(f.len(), 1);
    assert!(f[0].message.contains("Test serif") && f[0].severity == Severity::Blocks);
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 90]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
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

fn with_picture(alt: &str) -> Book {
    let mut b = gulliver();
    let ch = first_chapter(&mut b.body).unwrap();
    ch.content.insert(1, Block::Image { alt: alt.into(), src: "media/map.png".into(), role: ImageRole::Wide });
    b
}

#[test]
fn a_low_resolution_picture_is_a_warning_on_its_page() {
    let b = with_picture("A map");
    let pic = png(400, 200);
    let pr = build::print(&project(), &Default::default(), &b, &paperback(), &|_| Some(pic.clone())).unwrap();
    let pdf = pr.typeset.pdf().unwrap();
    let f = rules(&preflight::pdf_facts(&pr, &pdf), "pictures");
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].severity, Severity::Warning);
    let Link::Page(page) = f[0].link else { panic!("{:?}", f[0].link) };
    let chapter = pr.typeset.page_map().into_iter().find(|s| s.id == first_chapter(&mut b.clone().body).unwrap().id.to_string()).unwrap();
    assert!(page >= chapter.page && page <= chapter.page + 1, "page {page}, chapter at {}", chapter.page);
    // A sharp picture passes.
    let sharp = png(2400, 1200);
    let pr = build::print(&project(), &Default::default(), &b, &paperback(), &|_| Some(sharp.clone())).unwrap();
    assert!(rules(&preflight::pdf_facts(&pr, &pr.typeset.pdf().unwrap()), "pictures").is_empty());
}

#[test]
fn long_runs_of_blank_pages_block_export() {
    let f = preflight::blank_runs(&[false, true, true, true, false, true, false], 2, 10);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].link, Link::Page(2));
    assert!(f[0].message.contains("3 blank pages"));
    let end = preflight::blank_runs(&[false, true, true, true], 2, 2);
    assert_eq!(end.len(), 1);
    assert!(end[0].message.contains("at the end"));
    assert!(preflight::blank_runs(&[false, true, true, false, true, true, true], 2, 3).is_empty());
}

#[test]
fn a_short_book_is_blocked_and_an_odd_one_noted() {
    let mut b = gulliver();
    b.front.clear();
    b.back.clear();
    let ch = first_chapter(&mut b.body).unwrap().clone();
    let mut short = ch.clone();
    short.content.truncate(3);
    short.notes.clear();
    b.body = vec![short];
    let mut p = project();
    p.elements.clear();
    let pr = build::print(&p, &Default::default(), &b, &paperback(), &|_| None).unwrap();
    let f = preflight::page_count(&pr, wordsmith_book_model::Paper::Cream);
    assert!(f.iter().any(|x| x.severity == Severity::Blocks && x.message.contains("at least 24")), "{f:?}");
    if pr.typeset.page_count() % 2 == 1 {
        assert!(f.iter().any(|x| x.severity == Severity::Note));
    }
}

#[test]
fn a_print_edition_of_other_words_is_blocked_at_the_section() {
    let pr = gulliver_print(|_| {});
    let pdf = pr.typeset.pdf().unwrap();
    assert!(preflight::print_words(&pr, &pdf, &gulliver()).is_empty());
    // The source changed after the build: one word in the first chapter.
    let mut changed = gulliver();
    let ch = first_chapter(&mut changed.body).unwrap();
    let id = ch.id.to_string();
    if let Some(Block::Para(c)) = ch.content.get_mut(2)
        && let Some(wordsmith_manuscript_model::Inline::Span { text, .. }) = c.first_mut()
    {
        *text = format!("Lilliputian {text}");
    }
    let f = preflight::print_words(&pr, &pdf, &changed);
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].link, Link::Section(id));
    // And a PDF of another book.
    let other = build::print(&project(), &Default::default(), &waverley(), &paperback(), &|_| None).unwrap();
    let f = preflight::print_words(&pr, &other.typeset.pdf().unwrap(), &gulliver());
    assert!(f.iter().any(|x| x.rule == "words" && x.severity == Severity::Blocks));
}

fn ebook_bytes(b: &Book) -> Vec<u8> {
    build::ebook(&project(), &Default::default(), b, &Edition::ebook(), &|_| Some(png(800, 400))).unwrap().1.bytes
}

#[test]
fn ebook_rules_find_their_broken_ebooks() {
    let good = ebook_bytes(&gulliver());
    let f = preflight::ebook_findings(&good, &gulliver(), &[], true);
    assert!(f.is_empty(), "{f:?}");
    // A file missing from the package: an EPUB finding linked to the file.
    let mut files = wordsmith_book_check::epub::unzip(&good).unwrap();
    files.retain(|f| f.0 != "EPUB/text/s006.xhtml");
    let broken = rezip(&files);
    let f = preflight::ebook_findings(&broken, &gulliver(), &[], true);
    let e = rules(&f, "epub");
    assert!(!e.is_empty() && e.iter().all(|x| x.severity == Severity::Blocks));
    assert_eq!(e[0].link, Link::File("EPUB/package.opf".into()));
    // Accessibility metadata that does not match the content.
    let mut files = wordsmith_book_check::epub::unzip(&ebook_bytes(&with_picture("A map"))).unwrap();
    let opf = files.iter_mut().find(|f| f.0 == "EPUB/package.opf").unwrap();
    opf.2 = String::from_utf8(opf.2.clone()).unwrap().replace("<meta property=\"schema:accessMode\">visual</meta>", "").into_bytes();
    let f = preflight::accessibility(&rezip(&files));
    assert!(f.iter().any(|x| x.message.contains("visual access")), "{f:?}");
    // Words that differ from the source.
    let mut changed = gulliver();
    let ch = first_chapter(&mut changed.body).unwrap();
    if let Some(Block::Para(c)) = ch.content.get_mut(2)
        && let Some(wordsmith_manuscript_model::Inline::Span { text, .. }) = c.first_mut()
    {
        *text = format!("Brobdingnagian {text}");
    }
    assert!(rules(&preflight::ebook_findings(&good, &changed, &[], true), "words").len() == 1);
    // No cover, or a poor one: warnings.
    let f = preflight::ebook_findings(&good, &gulliver(), &["the cover is 500 × 800 pixels".into()], false);
    assert_eq!(rules(&f, "cover").len(), 2);
    assert!(rules(&f, "cover").iter().all(|x| x.severity == Severity::Warning));
}

fn rezip(files: &[(String, bool, Vec<u8>)]) -> Vec<u8> {
    use std::io::Write as _;
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, stored, bytes) in files {
        let m = if *stored { zip::CompressionMethod::Stored } else { zip::CompressionMethod::Deflated };
        z.start_file(name.as_str(), zip::write::SimpleFileOptions::default().compression_method(m)).unwrap();
        z.write_all(bytes).unwrap();
    }
    z.finish().unwrap().into_inner()
}

#[test]
fn missing_alt_text_blocks_the_ebook_at_its_section() {
    let mut src = gulliver_source();
    src.book = with_picture("");
    let id = first_chapter(&mut src.book.body).unwrap().id.to_string();
    let (r, bytes) = preflight::preflight(&project(), &Default::default(), &src, &Edition::ebook(), &|_| Some(png(800, 400)));
    assert!(bytes.is_none());
    let f = rules(&r.findings, "alt-text");
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].link, Link::Section(id));
    assert!(r.blocked());
}

#[test]
fn a_bad_isbn_blocks_at_its_edition_and_waiting_fixes_are_noted() {
    let src = gulliver_source();
    let mut p = project();
    let mut e = paperback();
    e.isbn = "978-0-306-40615-8".into();
    p.editions = vec![e.clone()];
    let ch = first_chapter(&mut src.book.clone().body).unwrap().clone();
    p.fixes.push(Fix {
        id: "f1".into(),
        section: Some(ch.id),
        find: "small estate".into(),
        replace: "small farm".into(),
        offered: true,
        made: "2026-11-24".into(),
        ..Fix::default()
    });
    let f = preflight::book_findings(&p, &src);
    let isbn: Vec<&Finding> = f.iter().filter(|x| x.rule == "metadata" && x.message.contains("ISBN")).collect();
    assert_eq!(isbn.len(), 1, "{f:?}");
    assert_eq!(isbn[0].severity, Severity::Blocks);
    assert_eq!(isbn[0].link, Link::Edition("paperback".into()));
    let fixes = rules(&f, "fixes");
    assert_eq!(fixes.len(), 1, "{f:?}");
    assert_eq!(fixes[0].severity, Severity::Note);
    assert!(fixes[0].message.starts_with("1 fix not yet accepted"));
    // The build uses the fix, and the words check counts it as the book's.
    let (r, _) = preflight::preflight(&p, &Default::default(), &src, &e, &|_| None);
    assert!(rules(&r.findings, "words").is_empty(), "{}", show(&r));
}
