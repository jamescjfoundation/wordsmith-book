// SPDX-License-Identifier: MIT OR Apache-2.0
//! The page list (plan R2.10, EBK-6): every print page has its marker in the ebook, in reading
//! order, where that page starts; the navigation's page list leads to each; the metadata says so.

mod common;

use common::*;
use wordsmith_book_check::{build, epub};
use wordsmith_book_model::Edition;

fn first_word(s: &str) -> String {
    s.split_whitespace().next().unwrap_or("").chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// The text after a position in XHTML, tags left out.
fn text_after(xhtml: &str, at: usize) -> String {
    let mut out = String::new();
    let mut tag = false;
    let at = at + xhtml[at..].find('>').map_or(0, |i| i + 1);
    for c in xhtml[at..].chars() {
        match c {
            '<' => tag = true,
            '>' => tag = false,
            c if !tag => out.push(c),
            _ => {}
        }
        if out.split_whitespace().count() > 3 {
            break;
        }
    }
    out
}

#[test]
fn every_print_page_has_its_marker_in_reading_order() {
    let p = project();
    let b = gulliver();
    let ebook = Edition::ebook();
    assert!(ebook.page_list);
    let (_, e) = build::ebook(&p, &Default::default(), &b, &ebook, &|_| None).unwrap();
    let paper = p.editions.iter().find(|e| e.format.is_print()).unwrap();
    let pr = build::print(&p, &Default::default(), &b, paper, &|_| None).unwrap();
    let pages = pr.typeset.page_count();
    assert!(epub::check(&e.bytes).is_empty(), "{:#?}", epub::check(&e.bytes));
    // In reading order: the spine's documents, each marker as it comes.
    let files = epub::unzip(&e.bytes).unwrap();
    let get = |href: &str| String::from_utf8(files.iter().find(|f| f.0 == format!("EPUB/{href}")).unwrap().2.clone()).unwrap();
    let mut found = Vec::new();
    let mut docs = Vec::new();
    for (id, _) in &e.package.spine {
        let item = e.package.items.iter().find(|i| &i.id == id).unwrap();
        let x = get(&item.href);
        for (at, _) in x.match_indices("epub:type=\"pagebreak\"") {
            let start = x[..at].rfind("id=\"page-").unwrap() + 9;
            let n: usize = x[start..].split('"').next().unwrap().parse().unwrap();
            found.push(n);
            docs.push((n, x.clone(), at));
        }
    }
    assert_eq!(found, (1..=pages).collect::<Vec<_>>(), "every page, once, in order");
    // The navigation's page list leads to each, labelled with its folio.
    assert_eq!(e.package.page_list.len(), pages);
    assert_eq!(e.package.page_list[0].label, pr.typeset.folio(0));
    let nav = get("nav.xhtml");
    assert!(nav.contains("epub:type=\"page-list\""));
    // The metadata.
    let meta = &e.package.metadata.meta;
    assert!(meta.iter().any(|(k, v)| k == "schema:accessibilityFeature" && v == "pageNavigation"));
    assert!(meta.iter().any(|(k, _)| k == "a11y:pageBreakSource"));
    // Where pages start in the author's text, the marker is at the page's first word.
    let pieces = jcj_layout::pages::body_pieces(pr.typeset.layout());
    let body: Vec<&jcj_doc::Paragraph> = jcj_doc::walk::body_paragraphs(pr.typeset.document()).into_iter().map(|(_, p)| p).collect();
    let mut checked = 0;
    let mut wrong = Vec::new();
    for (i, page) in pieces.iter().enumerate() {
        let Some(piece) =
            page.iter().find(|x| x.continued && body.get(x.block).is_some_and(|p| wordsmith_book_typeset::UNIT.get_or_none(*p).is_some()))
        else {
            continue;
        };
        if page.first().map(|x| x.block) != Some(piece.block) {
            continue;
        }
        let text = &body[piece.block].text;
        let from = piece.bytes.as_ref().map_or(0, |b| b.start);
        let from = text[..from].rfind(char::is_whitespace).map_or(0, |w| w + 1);
        let want = first_word(&text[from..]);
        let (_, x, at) = docs.iter().find(|(n, _, _)| *n == i + 1).unwrap();
        let got = first_word(&text_after(x, *at));
        checked += 1;
        if want != got {
            wrong.push((i + 1, want, got));
        }
    }
    assert!(checked > pages / 2, "{checked} of {pages}");
    assert!(wrong.len() * 50 <= checked, "markers off their page's first word: {wrong:?}");
}

#[test]
fn an_ebook_without_a_page_list_has_no_markers() {
    let mut e = Edition::ebook();
    e.page_list = false;
    let (_, eb) = build::ebook(&project(), &Default::default(), &gulliver(), &e, &|_| None).unwrap();
    assert!(eb.package.page_list.is_empty());
    let all: String = epub::unzip(&eb.bytes).unwrap().iter().map(|f| String::from_utf8_lossy(&f.2).into_owned()).collect();
    assert!(!all.contains("pagebreak") && !all.contains("pageNavigation"));
}
