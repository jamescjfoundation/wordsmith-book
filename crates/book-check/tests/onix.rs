// SPDX-License-Identifier: MIT OR Apache-2.0
//! Distributor metadata (plan R4.5, META-6): an ONIX for Books 3.0 message with one product per
//! edition validates against the published schema (with its code lists) where the validator is
//! installed (`BOOK_ONIX_VALIDATOR`: a program that takes the file and prints VALID), and carries
//! each edition's identifier, form, title, series, contributors, language, pages and imprint.

mod common;

use common::*;
use wordsmith_book_check::{build, onix};
use wordsmith_book_model::{Contributor, Edition, Imprint, Role};

fn validate(xml: &str, name: &str) {
    let Some(program) = std::env::var_os("BOOK_ONIX_VALIDATOR") else {
        eprintln!("no ONIX validator is set up (BOOK_ONIX_VALIDATOR); the schema check is skipped");
        return;
    };
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.xml"));
    std::fs::write(&path, xml).unwrap();
    let out = std::process::Command::new(program).arg(&path).output().unwrap();
    let report = String::from_utf8_lossy(&out.stdout);
    assert!(report.trim() == "VALID", "{name}:\n{report}");
}

#[test]
fn every_edition_has_a_record_that_validates() {
    let book = gulliver();
    let mut p = project();
    let mut paper = Edition::paperback();
    paper.isbn = "978-0-306-40615-7".into();
    let mut ebook = Edition::ebook();
    ebook.isbn = "978-1-86197-271-2".into();
    p.editions = vec![paper.clone(), ebook.clone()];
    p.metadata.contributors.push(Contributor { name: "A. Translator".into(), role: Role::Translator });
    p.metadata.imprint = Some(Imprint { name: "Lantern Press".into(), ..Imprint::default() });
    p.metadata.overrides.insert("description".into(), "A surgeon's voyages to remote nations & what he learned there.".into());
    p.metadata.overrides.insert("series".into(), "Voyages, Book 2".into());
    p.house.series_number = Some(2);
    let pages = build::print(&p, &Default::default(), &book, &paper, &|_| None).unwrap().typeset.page_count();
    let records = [onix::Record { edition: &paper, pages: Some(pages) }, onix::Record { edition: &ebook, pages: None }];
    let xml = onix::message(&p, &book, &records, "Lantern Press", "2026-11-25");
    assert_eq!(xml, onix::message(&p, &book, &records, "Lantern Press", "2026-11-25"), "the same bytes every time");
    validate(&xml, "onix-full");
    let doc = roxmltree::Document::parse(&xml).unwrap();
    let products: Vec<_> = doc.descendants().filter(|n| n.has_tag_name("Product")).collect();
    assert_eq!(products.len(), 2);
    let text = |n: &roxmltree::Node, tag: &str| n.descendants().find(|d| d.has_tag_name(tag)).and_then(|d| d.text()).unwrap_or("").to_owned();
    assert_eq!((text(&products[0], "IDValue"), text(&products[0], "ProductForm")), ("9780306406157".to_owned(), "BC".to_owned()));
    assert_eq!(
        (text(&products[1], "IDValue"), text(&products[1], "ProductForm"), text(&products[1], "ProductFormDetail")),
        ("9781861972712".to_owned(), "ED".to_owned(), "E101".to_owned())
    );
    assert_eq!(text(&products[0], "ExtentValue"), pages.to_string());
    assert!(text(&products[1], "ExtentValue").is_empty(), "an ebook has no page count");
    assert_eq!(text(&products[0], "LanguageCode"), "eng");
    assert_eq!(text(&products[0], "ImprintName"), "Lantern Press");
    assert_eq!(text(&products[0], "PartNumber"), "2");
    let roles: Vec<String> =
        products[0].descendants().filter(|n| n.has_tag_name("ContributorRole")).filter_map(|n| n.text().map(str::to_owned)).collect();
    assert_eq!(roles, ["A01", "B06"]);
    assert!(xml.contains("&amp; what he learned"), "text is escaped");
}

#[test]
fn a_book_without_isbns_or_details_still_validates() {
    let book = gulliver();
    let mut p = project();
    p.metadata.contributors.clear();
    // No imprint or publisher anywhere: the author publishes it.
    let mut book = book;
    book.metadata.info.publisher.clear();
    let e = Edition::ebook();
    let xml = onix::message(&p, &book, &[onix::Record { edition: &e, pages: None }], "", "2026-11-25");
    validate(&xml, "onix-bare");
    assert!(xml.contains("<ProductIDType>01</ProductIDType>"), "the edition's own identifier");
    assert!(xml.contains("<PublisherName>Jonathan Swift</PublisherName>") && !xml.contains("<Imprint>"));
    assert_eq!(onix::language_code("pt-BR"), "por");
    assert_eq!(onix::language_code("xx"), "und");
}
