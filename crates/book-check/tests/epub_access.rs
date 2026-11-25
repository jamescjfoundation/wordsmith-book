// SPDX-License-Identifier: MIT OR Apache-2.0
//! Ebook accessibility, fonts and cover (plan M6.2, EBK-3, EBK-4, EBK-5): the accessibility
//! metadata matches what the ebook holds; a picture without alternative text blocks export
//! unless it is marked decorative; fonts are in the package when the edition embeds them and
//! not otherwise; the cover is declared, read first and checked.

mod common;

use std::io::Read as _;

use common::*;
use wordsmith_book_check::build;
use wordsmith_book_epub::Error;
use wordsmith_book_model::{BookProject, Edition};
use wordsmith_manuscript_format::book::{Book, Section};
use wordsmith_manuscript_model::{Block, ImageRole, Role};

fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 90]));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

fn first_chapter(sections: &mut [Section]) -> Option<&mut Section> {
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

/// The test book with a picture in its first chapter, with `alt`.
fn with_picture(alt: &str) -> Book {
    let mut b = gulliver();
    let ch = first_chapter(&mut b.body).unwrap();
    ch.content.insert(1, Block::Image { alt: alt.into(), src: "media/map.png".into(), role: ImageRole::Wide });
    b
}

fn files(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    (0..z.len())
        .map(|i| {
            let mut f = z.by_index(i).unwrap();
            let mut b = Vec::new();
            f.read_to_end(&mut b).unwrap();
            (f.name().to_owned(), b)
        })
        .collect()
}

fn text(files: &[(String, Vec<u8>)], path: &str) -> String {
    String::from_utf8(files.iter().find(|(p, _)| p == path).unwrap_or_else(|| panic!("no {path}")).1.clone()).unwrap()
}

fn build_with(p: &BookProject, b: &Book, e: &Edition) -> Result<wordsmith_book_epub::Ebook, String> {
    let pic = png(800, 400);
    let cover = png(1600, 2560);
    let small = png(500, 800);
    build::ebook(p, &Default::default(), b, e, &|path| match path {
        "media/map.png" => Some(pic.clone()),
        "assets/cover.png" => Some(cover.clone()),
        "assets/small.png" => Some(small.clone()),
        _ => None,
    })
    .map(|(_, e)| e)
}

fn meta(e: &wordsmith_book_epub::Ebook, prop: &str) -> Vec<String> {
    e.package.metadata.meta.iter().filter(|(k, _)| k == prop).map(|(_, v)| v.clone()).collect()
}

#[test]
fn accessibility_metadata_follows_what_the_ebook_holds() {
    let p = project();
    let plain = build_with(&p, &gulliver(), &Edition::ebook()).unwrap();
    assert_eq!(meta(&plain, "schema:accessMode"), vec!["textual"]);
    assert_eq!(meta(&plain, "schema:accessModeSufficient"), vec!["textual"]);
    let features = meta(&plain, "schema:accessibilityFeature");
    for f in ["structuralNavigation", "tableOfContents", "readingOrder"] {
        assert!(features.iter().any(|x| x == f), "{f}");
    }
    assert!(!features.iter().any(|x| x == "alternativeText"));
    // A new ebook edition carries the print edition's pages (R2.10).
    assert!(features.iter().any(|x| x == "pageNavigation"));
    assert_eq!(meta(&plain, "schema:accessibilityHazard"), vec!["none"]);
    assert_eq!(meta(&plain, "dcterms:conformsTo"), vec!["EPUB Accessibility 1.1 - WCAG 2.2 Level AA"]);
    assert_eq!(meta(&plain, "schema:accessibilitySummary").len(), 1);
    assert!(!meta(&plain, "a11y:certifiedBy").is_empty());
    // A picture with alternative text: visual content, and the feature that describes it.
    let pictured = build_with(&p, &with_picture("A map of the island"), &Edition::ebook()).unwrap();
    assert_eq!(meta(&pictured, "schema:accessMode"), vec!["textual", "visual"]);
    assert!(meta(&pictured, "schema:accessibilityFeature").iter().any(|x| x == "alternativeText"));
    let f = files(&pictured.bytes);
    let all: String = f.iter().filter(|(p, _)| p.ends_with(".xhtml")).map(|(_, b)| String::from_utf8_lossy(b).into_owned()).collect();
    assert!(all.contains("alt=\"A map of the island\""));
    assert!(f.iter().any(|(p, _)| p.starts_with("EPUB/images/") && p.ends_with(".png")));
    // The package document says it all.
    let opf = text(&f, "EPUB/package.opf");
    assert!(opf.contains("<meta property=\"schema:accessMode\">visual</meta>"));
    assert!(opf.contains("prefix=\"schema: http://schema.org/"));
}

#[test]
fn a_picture_without_alternative_text_blocks_export_unless_decorative() {
    let mut p = project();
    let b = with_picture("  ");
    let err = build::ebook(&p, &Default::default(), &b, &Edition::ebook(), &|_| Some(png(10, 10))).unwrap_err();
    assert!(err.contains("media/map.png") && err.contains("alternative text"), "{err}");
    let assembled = assembled(&b);
    assert_eq!(wordsmith_book_epub::access::missing_alt_text(&assembled, &[]), vec!["media/map.png".to_owned()]);
    // Marked decorative: exported, hidden from assistive technology, and not visual content.
    p.decorative.push("media/map.png".into());
    let e = build_with(&p, &b, &Edition::ebook()).unwrap();
    let all: String = files(&e.bytes).iter().filter(|(p, _)| p.ends_with(".xhtml")).map(|(_, b)| String::from_utf8_lossy(b).into_owned()).collect();
    assert!(all.contains("role=\"presentation\"><img src=\"../images/i001.png\" alt=\"\"/>"));
    assert_eq!(meta(&e, "schema:accessMode"), vec!["textual"]);
    let lib = wordsmith_book_epub::ebook(&assembled, &classic(), &Edition::ebook(), &wordsmith_book_epub::Settings::default(), &|_| None);
    assert!(matches!(lib, Err(Error::NoAltText(v)) if v.len() == 1));
}

#[test]
fn fonts_are_embedded_when_the_edition_says_and_only_then() {
    let p = project();
    let with = build_with(&p, &gulliver(), &Edition::ebook()).unwrap();
    assert!(with.fonts_refused.is_empty());
    let f = files(&with.bytes);
    let fonts: Vec<&String> = f.iter().map(|(p, _)| p).filter(|p| p.starts_with("EPUB/fonts/")).collect();
    assert!(fonts.len() >= 4, "{fonts:?}");
    let css = text(&f, "EPUB/css/book.css");
    assert!(css.contains("@font-face") && css.contains("font-family: \"book-classic-serif\", serif"));
    for font in &fonts {
        let href = font.trim_start_matches("EPUB/");
        assert!(css.contains(&format!("url(\"../{href}\")")), "{href}");
    }
    assert!(text(&f, "EPUB/package.opf").contains("media-type=\"font/ttf\""));
    let mut off = Edition::ebook();
    off.embed_fonts = false;
    let without = build_with(&p, &gulliver(), &off).unwrap();
    let f = files(&without.bytes);
    assert!(!f.iter().any(|(p, _)| p.starts_with("EPUB/fonts/")));
    assert!(!text(&f, "EPUB/css/book.css").contains("@font-face"));
    // Every design embeds its body font, and its heading font when it uses one.
    for d in wordsmith_book_design::builtin::all() {
        let (items, css, refused) = wordsmith_book_epub::fonts::embedded(&d);
        assert!(refused.is_empty() && !items.is_empty() && css.body.is_some(), "{}", d.id);
    }
}

#[test]
fn the_cover_is_declared_read_first_and_checked() {
    let p = project();
    let mut e = Edition::ebook();
    e.cover = "assets/cover.png".into();
    let book = build_with(&p, &gulliver(), &e).unwrap();
    assert!(book.cover_problems.is_empty(), "{:?}", book.cover_problems);
    let f = files(&book.bytes);
    let opf = text(&f, "EPUB/package.opf");
    assert!(opf.contains("href=\"images/cover.png\" media-type=\"image/png\" properties=\"cover-image\""));
    let first = opf.split("<itemref idref=\"").nth(1).unwrap();
    assert!(first.starts_with("cover\""), "the cover is read first");
    let page = text(&f, "EPUB/text/cover.xhtml");
    assert!(page.contains("epub:type=\"cover\"") && page.contains("alt=\"Cover of "));
    assert!(text(&f, "EPUB/nav.xhtml").contains("epub:type=\"cover\" href=\"text/cover.xhtml#cover\""));
    // A small cover is built, with its problems reported.
    e.cover = "assets/small.png".into();
    let small = build_with(&p, &gulliver(), &e).unwrap();
    assert_eq!(small.cover_problems.len(), 1, "{:?}", small.cover_problems);
    assert!(small.cover_problems[0].contains("1600"));
    assert!(wordsmith_book_epub::cover::problems(&png(1000, 1000)).len() == 2);
    // A missing cover file stops the build.
    e.cover = "assets/none.png".into();
    assert!(build_with(&p, &gulliver(), &e).unwrap_err().contains("assets/none.png"));
}
