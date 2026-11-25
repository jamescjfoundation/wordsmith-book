// SPDX-License-Identifier: MIT OR Apache-2.0
//! Typographic warnings (plan R2.13, PRE-3): each problem found on a page written to show it,
//! and not on the test book set as its design says.

mod common;

use common::*;
use wordsmith_book_check::{build, preflight};
use wordsmith_book_model::Edition;
use wordsmith_book_typeset::{Block, Kind, PrintBook, Section, Span, TypoKind, typeset};

fn one_chapter(paras: Vec<String>) -> PrintBook {
    PrintBook {
        title: "Test".into(),
        author: "Someone".into(),
        lang: "en".into(),
        sections: vec![Section {
            id: "c".into(),
            kind: Kind::Chapter,
            title: "One".into(),
            blocks: paras.into_iter().map(|p| Block::Para(vec![Span::plain(p)])).collect(),
            ..Section::default()
        }],
    }
}

fn kinds(t: &wordsmith_book_typeset::Typeset, limit: usize) -> Vec<TypoKind> {
    t.typography(limit, 0.6).into_iter().map(|i| i.kind).collect()
}

#[test]
fn widows_and_orphans_are_found_when_the_design_allows_them_and_not_otherwise() {
    let (pb, mut look, _) = print(&gulliver(), &classic());
    let kept = typeset(&pb, &look, "2026-11-24").unwrap();
    assert!(!kinds(&kept, 2).iter().any(|k| matches!(k, TypoKind::Widow | TypoKind::Orphan)), "widow control is on");
    look.widow_control = false;
    let loose = typeset(&pb, &look, "2026-11-24").unwrap();
    let found = loose.typography(2, 0.6);
    let lone: Vec<_> = found.iter().filter(|t| matches!(t.kind, TypoKind::Widow | TypoKind::Orphan)).collect();
    assert!(!lone.is_empty(), "with control off, some page has a lone line");
    assert!(lone.iter().all(|t| t.page >= 1 && t.page <= loose.page_count() && !t.text.is_empty()));
}

#[test]
fn stacked_hyphens_loose_lines_and_rivers_are_found_on_pages_written_to_show_them() {
    let (_, mut look, _) = print(&gulliver(), &classic());
    // A narrow measure of long words with soft hyphens: hyphens stack.
    look.trim = (3.2, 6.0);
    look.margins = (0.6, 0.6, 0.6, 0.6);
    look.gutter.clear();
    let soft =
        "in\u{AD}com\u{AD}pre\u{AD}hen\u{AD}si\u{AD}bil\u{AD}i\u{AD}ty in\u{AD}ter\u{AD}na\u{AD}tion\u{AD}al\u{AD}i\u{AD}za\u{AD}tion ".repeat(30);
    let t = typeset(&one_chapter(vec![soft]), &look, "2026-11-24").unwrap();
    assert!(kinds(&t, 1).contains(&TypoKind::StackedHyphens), "{:?}", kinds(&t, 1));
    // Medium words on the narrow measure, justified, without hyphens: loose lines.
    look.hyphenate = false;
    let medium = "remarkable extraordinary beautifully considerable unfortunately ".repeat(30);
    let t = typeset(&one_chapter(vec![medium]), &look, "2026-11-24").unwrap();
    assert!(kinds(&t, 2).contains(&TypoKind::LooseLine), "{:?}", kinds(&t, 2));
    // The same short words on every line, justified on a wide measure: spaces line up.
    let (_, mut wide, _) = print(&gulliver(), &classic());
    wide.hyphenate = false;
    let same = "aaa bbb ccc ddd eee fff ggg hhh iii jjj kkk ".repeat(40);
    let t = typeset(&one_chapter(vec![same]), &wide, "2026-11-24").unwrap();
    assert!(kinds(&t, 2).contains(&TypoKind::River), "{:?}", kinds(&t, 2));
}

#[test]
fn preflight_lists_them_by_page() {
    let mut p = project();
    p.design.id = "classic".into();
    let mut pr = build::print(&p, &Default::default(), &gulliver(), &Edition::paperback(), &|_| None).unwrap();
    pr.look.widow_control = false;
    pr.typeset = typeset(&pr.book, &pr.look, "2026-11-24").unwrap();
    let f = preflight::typography(&pr);
    let w: Vec<_> = f.iter().filter(|x| x.severity == wordsmith_book_model::Severity::Warning).collect();
    assert!(!w.is_empty());
    assert!(w.iter().all(|x| matches!(x.link, preflight::Link::Page(_)) && x.message.starts_with("Page ")));
}
