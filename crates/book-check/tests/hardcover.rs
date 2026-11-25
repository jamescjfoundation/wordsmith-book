// SPDX-License-Identifier: MIT OR Apache-2.0
//! Hardcover (plan R2.2, PRT-8, PRE-4): its profile's case-laminate sizes, wider margins and page
//! limits, against the paperback's; the test book as a hardcover passes preflight.

mod common;

use common::*;
use wordsmith_book_check::{build, preflight, profile};
use wordsmith_book_format::source;
use wordsmith_book_model::{Edition, Format, Paper, Severity, Source};

#[test]
fn margins_and_limits_per_profile() {
    let hard = profile::get("pod-hardcover").unwrap();
    let soft = profile::get("pod-paperback").unwrap();
    assert_eq!(Format::Hardcover.default_profile(), "pod-hardcover");
    assert_eq!(Edition::hardcover().profile_id(), "pod-hardcover");
    assert_eq!(Edition { profile: String::new(), ..Edition::hardcover() }.profile_id(), "pod-hardcover");
    // Every case-laminate size is also a paperback size, with every margin wider.
    for t in &hard.trims {
        let p = soft.trim(&t.id).unwrap_or_else(|| panic!("{}", t.id));
        for i in 0..4 {
            assert!(t.margins[i] > p.margins[i], "{}: margin {i}", t.id);
        }
    }
    // The gutter is wider at every page count.
    for pages in [100, 200, 400, 600, 800] {
        assert!(hard.gutter_for(pages) > soft.gutter_for(pages), "{pages} pages");
    }
    assert!(hard.min_pages > soft.min_pages && hard.max_pages.of(Paper::Cream) < soft.max_pages.of(Paper::Cream));
    assert!(hard.minimum.outside > soft.minimum.outside);
}

#[test]
fn the_test_book_as_a_hardcover_passes_and_a_short_one_does_not() {
    let src = source::open(&novels(), &Source::Manuscript("gullivers-travels.wsm".into())).unwrap();
    let p = project();
    let e = Edition::hardcover();
    let (r, pdf) = preflight::preflight(&p, &Default::default(), &src, &e, &|_| None);
    assert!(!r.blocked(), "{:#?}", r.of(Severity::Blocks));
    assert!(pdf.is_some());
    let pr = build::print(&p, &Default::default(), &gulliver(), &e, &|_| None).unwrap();
    let pages = pr.typeset.page_count();
    assert!((pr.typeset.gutter() - pr.profile.gutter_for(pages)).abs() < 1e-4);
    assert_eq!(pr.profile.id, "pod-hardcover");
    // Fewer pages than a case can bind: blocked; the same as a paperback: fine.
    let mut b = gulliver();
    b.front.clear();
    b.back.clear();
    b.body.truncate(1);
    let mut short = p.clone();
    short.elements.clear();
    let pr = build::print(&short, &Default::default(), &b, &e, &|_| None).unwrap();
    let n = pr.typeset.page_count();
    assert!((24..75).contains(&n), "{n} pages");
    assert!(preflight::page_count(&pr, Paper::Cream).iter().any(|f| f.severity == Severity::Blocks && f.message.contains("at least 75")));
    let pr = build::print(&short, &Default::default(), &b, &Edition::paperback(), &|_| None).unwrap();
    assert!(!preflight::page_count(&pr, Paper::Cream).iter().any(|f| f.severity == Severity::Blocks));
}
