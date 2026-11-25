// SPDX-License-Identifier: MIT OR Apache-2.0
//! Asides (plan R2.6, TXT-2): letters, notes, signs, messages and other kinds, each set as the
//! design says, in print and in the ebook, every word kept.

mod common;

use common::*;
use wordsmith_book_check::{build, epub, preflight, verify};
use wordsmith_book_design::design::AsideStyle;
use wordsmith_book_model::Edition;
use wordsmith_manuscript_model::{Block, Fence, Inline, Role};

const KINDS: [(&str, &str); 6] = [
    ("letter", "My dear cousin, I write in haste from the harbour."),
    ("note", "Back by noon. Feed the cat."),
    ("sign", "No admittance except on business."),
    ("message", "Arriving Tuesday stop meet the coach stop."),
    ("", "An aside of no particular kind."),
    ("diary", "Rain again; the ship has not come."),
];

fn with_asides() -> wordsmith_manuscript_format::book::Book {
    fn first(s: &mut [wordsmith_manuscript_format::book::Section]) -> Option<&mut wordsmith_manuscript_format::book::Section> {
        for x in s {
            if x.role == Role::Chapter && !x.content.is_empty() {
                return Some(x);
            }
            if let Some(c) = first(&mut x.children) {
                return Some(c);
            }
        }
        None
    }
    let mut b = gulliver();
    let ch = first(&mut b.body).unwrap();
    for (i, (kind, text)) in KINDS.iter().enumerate() {
        let kind = (!kind.is_empty()).then(|| (*kind).to_owned());
        ch.content.insert(
            2 + i,
            Block::Fenced { kind: Fence::Aside(kind), paras: vec![vec![Inline::Span { text: (*text).into(), marks: Default::default() }]] },
        );
    }
    b
}

#[test]
fn each_aside_kind_is_set_as_the_design_says_in_print() {
    let b = with_asides();
    let d = classic();
    let pr = build::print(&project(), &Default::default(), &b, &Edition::paperback(), &|_| None).unwrap();
    let paras = jcj_doc::walk::body_paragraphs(pr.typeset.document());
    for (kind, text) in KINDS {
        let look = d.asides.of(kind);
        let (_, p) = paras.iter().find(|(_, p)| p.text == text).unwrap_or_else(|| panic!("{kind}: not printed"));
        let c = &p.runs[0].props;
        match look.style {
            AsideStyle::Italic => assert_eq!(c.italic, Some(true), "{kind}"),
            AsideStyle::Centered => assert_eq!(p.props.align, Some(jcj_doc::props::Align::Center), "{kind}"),
            AsideStyle::Framed => assert!(p.props.borders.as_ref().is_some_and(|b| b.top.is_some() && b.bottom.is_some()), "{kind}"),
            AsideStyle::Inset => assert!(p.props.indent_left.is_some_and(|i| i > 0.0) && c.italic != Some(true), "{kind}"),
        }
        assert_eq!(c.small_caps == Some(true), look.small_caps, "{kind}");
        assert!((c.size.unwrap() - d.body.size * look.size).abs() < 0.01, "{kind}");
    }
    // Unknown and empty kinds take the default look.
    assert_eq!(d.asides.of("diary"), d.asides.of(""));
    let pdf = pr.typeset.pdf().unwrap();
    assert!(preflight::print_words(&pr, &pdf, &b).is_empty());
}

#[test]
fn each_aside_kind_is_marked_and_styled_in_the_ebook() {
    let b = with_asides();
    let (_, e) = build::ebook(&project(), &Default::default(), &b, &Edition::ebook(), &|_| None).unwrap();
    assert!(epub::check(&e.bytes).is_empty(), "{:#?}", epub::check(&e.bytes));
    let files = epub::unzip(&e.bytes).unwrap();
    let all: String = files.iter().filter(|f| f.0.ends_with(".xhtml")).map(|f| String::from_utf8_lossy(&f.2).into_owned()).collect();
    for (class, kind) in [("aside-italic", "letter"), ("aside-centered", "sign"), ("aside-framed", "message"), ("aside-inset", "note")] {
        assert!(all.contains(&format!("aside {class}")) && all.contains(&format!("aside-kind-{kind}")), "{kind}");
    }
    let css = String::from_utf8(files.iter().find(|f| f.0.ends_with("book.css")).unwrap().2.clone()).unwrap();
    for class in ["aside-italic", "aside-framed", "aside-centered", "aside-sans"] {
        assert!(css.contains(class), "{class}");
    }
    verify::compare(&verify::book_words(&b, true), &epub::words(&e.bytes, true).unwrap()).unwrap();
}

#[test]
fn a_design_can_restyle_an_aside_kind_and_bad_values_are_refused() {
    let mut d = classic();
    d.asides.sign.style = AsideStyle::Framed;
    d.asides.sign.font = "no-such-role".into();
    d.asides.letter.size = 3.0;
    let text = wordsmith_book_design::design::write(&d).unwrap();
    assert!(text.contains("[asides.sign]"), "{text}");
    let problems = wordsmith_book_design::design::problems(&d, &|r| wordsmith_book_design::fonts::Role::from_key(r).is_some());
    assert!(problems.iter().any(|p| p.contains("sign aside's font")) && problems.iter().any(|p| p.contains("letter aside size")), "{problems:?}");
}
