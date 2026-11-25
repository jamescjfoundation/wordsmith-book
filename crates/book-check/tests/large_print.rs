// SPDX-License-Identifier: MIT OR Apache-2.0
//! The large-print variant (plan R3.2, PRT-11): one click (a large-print edition) sets the book
//! in the large-print variant of whatever design it has: every variant validates, all its text
//! is 16 pt and up, and the test book passes preflight in each; a large-print edition with
//! smaller text is blocked.

mod common;

use common::*;
use wordsmith_book_check::{build, preflight};
use wordsmith_book_design::builtin;
use wordsmith_book_design::design::{FirstLine, Hyphenation, problems};
use wordsmith_book_design::large_print::{LARGE_PRINT_HEADS, LARGE_PRINT_SIZE, variant};
use wordsmith_book_format::source;
use wordsmith_book_model::{Edition, Format, Severity, Source};

fn large_print() -> Edition {
    let mut e = Edition::paperback();
    e.id = "large-print".into();
    e.name = "Large print".into();
    e.format = Format::LargePrint;
    e.trim = "6x9".into();
    e
}

#[test]
fn every_design_has_a_valid_large_variant() {
    for d in builtin::all() {
        let v = variant(&d);
        assert!(problems(&v, &wordsmith_book_design::known_role).is_empty(), "{}: {:?}", d.id, problems(&v, &wordsmith_book_design::known_role));
        assert!(v.body.size >= LARGE_PRINT_SIZE && v.heads.size >= LARGE_PRINT_HEADS, "{}", d.id);
        assert!(v.chapter.title_size >= LARGE_PRINT_SIZE && v.chapter.number_size >= LARGE_PRINT_SIZE, "{}", d.id);
        assert!(!v.body.justify && v.body.hyphenation == Hyphenation::Off && v.chapter.first_line != FirstLine::DropCap, "{}", d.id);
        assert_eq!(v.id, format!("{}-large-print", d.id));
        // Its character stays: the same fonts, marks and alignment.
        assert_eq!(
            (v.body.font.as_str(), v.chapter.font.as_str(), v.scene_break.mark.as_str()),
            (d.body.font.as_str(), d.chapter.font.as_str(), d.scene_break.mark.as_str())
        );
        // Proportions kept where the limits allow.
        if d.body.size < LARGE_PRINT_SIZE && d.chapter.title_size * LARGE_PRINT_SIZE / d.body.size <= 72.0 {
            let k = LARGE_PRINT_SIZE / d.body.size;
            assert!((v.chapter.title_size - d.chapter.title_size * k).abs() <= 0.5, "{}", d.id);
        }
    }
}

#[test]
fn the_test_book_in_large_print_is_16_pt_and_up_and_passes_preflight_in_every_design() {
    let src = source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap();
    for d in builtin::all() {
        let mut p = project();
        p.design.id = d.id.clone();
        let e = large_print();
        let pr = build::print(&p, &Default::default(), &src.book, &e, &|_| None).unwrap();
        assert_eq!(pr.look.min_size, LARGE_PRINT_SIZE);
        let text_only = wordsmith_book_typeset::Look { heads: wordsmith_book_typeset::Heads::None, folios: false, ..pr.look.clone() };
        let (size, at) = pr.typeset.smallest_type(&text_only).unwrap();
        assert!(size >= LARGE_PRINT_SIZE - 0.01, "{}: {size} pt at “{at}”", d.id);
        assert!(pr.look.head_size >= LARGE_PRINT_HEADS, "{}", d.id);
        let (r, bytes) = preflight::preflight(&p, &Default::default(), &src, &e, &|_| None);
        assert!(bytes.is_some());
        let blocks: Vec<String> = r.of(Severity::Blocks).iter().map(|f| format!("{}: {}", f.rule, f.message)).collect();
        assert!(blocks.is_empty(), "{}: {blocks:#?}", d.id);
    }
}

#[test]
fn a_large_print_edition_with_smaller_text_is_blocked() {
    let src = source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap();
    let p = project();
    // The book's own design (11 pt) laid out, then held to large print's floor.
    let mut pr = build::print(&p, &Default::default(), &src.book, &Edition::paperback(), &|_| None).unwrap();
    pr.look.min_size = LARGE_PRINT_SIZE;
    let pdf = pr.typeset.pdf().unwrap();
    let f = preflight::print_findings(&pr, &pdf, &src.book, &large_print());
    assert!(f.iter().any(|x| x.rule == "type-size" && x.severity == Severity::Blocks && x.message.contains("large print")), "{f:#?}");
}
