// SPDX-License-Identifier: MIT OR Apache-2.0
//! All common trims and metric sizes (plan R2.1, PRT-1): each trim of each profile lays the test
//! book out at its size, with its margins and the gutter the page count calls for.

mod common;

use common::*;
use wordsmith_book_check::{build, profile};
use wordsmith_book_model::Edition;

#[test]
fn every_trim_gives_its_page_size_margins_and_gutter() {
    let b = gulliver();
    for id in ["pod-paperback", "generic-pdf"] {
        let prof = profile::get(id).unwrap();
        assert!(prof.trims.len() >= 18, "{id}");
        assert!(prof.trims.iter().filter(|t| t.name.contains("mm")).count() >= 5, "{id}: metric sizes");
        for t in &prof.trims {
            let mut e = Edition::paperback();
            e.trim = t.id.clone();
            e.profile = id.into();
            // The whole book for the print-on-demand profile (its gutter follows the page count);
            // the opening for the generic one (no gutter table).
            let pr = if id == "pod-paperback" {
                build::print(&project(), &Default::default(), &b, &e, &|_| None)
            } else {
                build::print_opening(&project(), &Default::default(), &b, &e, &|_| None)
            }
            .unwrap_or_else(|err| panic!("{id} {}: {err}", t.id));
            let ty = &pr.typeset;
            let (w, h) = ty.page_size(0).unwrap();
            assert!((w - t.width * 72.0).abs() < 0.05 && (h - t.height * 72.0).abs() < 0.05, "{id} {}: {w} × {h}", t.id);
            let pages = ty.page_count();
            let gutter = prof.gutter_for(pages);
            assert!((ty.gutter() - gutter).abs() < 1e-4, "{id} {}: gutter {} for {pages} pages, want {gutter}", t.id, ty.gutter());
            // Page 1 is a right-hand page: inside margin on the left.
            let (x, y, bw, _) = ty.body_area(0).unwrap();
            let inside = x / 72.0;
            let outside = (w - x - bw) / 72.0;
            assert!((inside - (t.margins[0] + gutter)).abs() < 0.01, "{id} {}: inside {inside}", t.id);
            assert!((outside - t.margins[1]).abs() < 0.01, "{id} {}: outside {outside}", t.id);
            assert!(y / 72.0 + 0.01 >= t.margins[2], "{id} {}: top", t.id);
            assert!(t.margins[1] >= prof.minimum.outside && t.margins[2] >= prof.minimum.top && t.margins[3] >= prof.minimum.bottom, "{id} {}", t.id);
        }
    }
}
