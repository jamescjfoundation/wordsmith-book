// SPDX-License-Identifier: MIT OR Apache-2.0
//! Pages (plan M5.2): page sizes per trim, the gutter on each side of every threshold, right-hand
//! starts, the folio sequence, no running head on openings or blank pages, the three styles of
//! running heads, blank-page runs within the profile's limits, and a layout loop that settles.

mod common;

use common::*;
use wordsmith_book_check::build;
use wordsmith_book_check::profile;
use wordsmith_book_design::builtin;
use wordsmith_book_model::Edition;
use wordsmith_book_typeset::{Kind, MAX_PASSES};

fn gulliver_print(design: &str, trim: &str) -> build::Print {
    let mut p = project();
    p.design.id = design.into();
    let mut e = Edition::paperback();
    e.trim = trim.into();
    build::print(&p, &Default::default(), &gulliver(), &e, &|_| None).unwrap()
}

/// Each page's text above the body (heads) and below it (feet), and whether it prints anything.
fn bands(t: &wordsmith_book_typeset::Typeset) -> Vec<(String, String, usize)> {
    let pdf = t.pdf().unwrap();
    let pages = jcj_output::read_pages(&pdf, &jcj_output::ReadLimits { max_input_bytes: 256 << 20, max_pages: 10_000, max_glyphs_per_page: 20_000 }).unwrap();
    let boxes = jcj_layout::page_boxes(t.layout());
    pages
        .iter()
        .zip(&boxes)
        .map(|(p, b)| {
            let (mut head, mut foot) = (String::new(), String::new());
            for g in &p.glyphs {
                let y = f64::from(b.page.h) - g.y;
                let text = g.text.clone().unwrap_or_default();
                if y < f64::from(b.body.y) - 3.0 {
                    head.push_str(&text);
                } else if y > f64::from(b.body.y + b.body.h) + 3.0 {
                    foot.push_str(&text);
                }
            }
            (head.trim().to_owned(), foot.trim().to_owned(), p.glyphs.iter().filter(|g| g.text.as_deref().is_some_and(|t| !t.trim().is_empty())).count())
        })
        .collect()
}

#[test]
fn every_trim_gives_its_page_size() {
    let prof = profile::get("pod-paperback").unwrap();
    assert_eq!(prof.trims.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["5x8", "5.25x8", "5.5x8.5", "6x9"]);
    for t in &prof.trims {
        let pr = gulliver_print("classic", &t.id);
        let n = pr.typeset.page_count();
        for i in [0, n / 2, n - 1] {
            let (w, h) = pr.typeset.page_size(i).unwrap();
            assert!((w - t.width * 72.0).abs() < 0.01 && (h - t.height * 72.0).abs() < 0.01, "{}: page {i} is {w} × {h}", t.id);
        }
        // The gutter the build settled on is the profile's for the page count it has.
        assert_eq!(pr.typeset.gutter(), prof.gutter_for(n), "{}: {n} pages", t.id);
        assert!(pr.typeset.passes() <= MAX_PASSES);
    }
}

#[test]
fn the_gutter_steps_at_every_threshold() {
    let prof = profile::get("pod-paperback").unwrap();
    for (i, g) in prof.gutter.iter().enumerate() {
        assert_eq!(prof.gutter_for(g.pages), g.inches, "at {} pages", g.pages);
        if let Some(next) = prof.gutter.get(i + 1) {
            assert_eq!(prof.gutter_for(g.pages + 1), next.inches, "at {} pages", g.pages + 1);
        }
        if g.pages > 1 {
            assert_eq!(prof.gutter_for(g.pages - 1), g.inches);
        }
    }
    assert_eq!(prof.gutter_for(1), 0.375);
    assert_eq!(prof.gutter_for(1_000_000), 0.875);
    // The look the build uses carries the same table.
    let pr = gulliver_print("classic", "6x9");
    for g in &prof.gutter {
        assert_eq!(pr.look.gutter_for(g.pages), g.inches);
    }
}

#[test]
fn chapters_open_on_right_hand_pages_without_heads_and_folios_run_in_order() {
    let pr = gulliver_print("classic", "6x9");
    let t = &pr.typeset;
    let map = t.page_map();
    let bands = bands(t);
    for (s, start) in pr.book.sections.iter().zip(&map) {
        if s.recto {
            assert_eq!(start.page % 2, 1, "{} opens on page {}", s.id, start.page);
        }
        if matches!(s.kind, Kind::Chapter | Kind::Part) {
            assert!(bands[start.page - 1].0.is_empty(), "no running head on the opening of {} (page {})", s.id, start.page);
        }
    }
    // Blank pages print nothing at all: no head, no folio. Runs of them stay within the profile.
    let mut run = 0;
    let mut worst = 0;
    for (i, (head, foot, glyphs)) in bands.iter().enumerate() {
        if *glyphs == 0 {
            run += 1;
            worst = worst.max(run);
            assert!(head.is_empty() && foot.is_empty(), "page {} is blank", i + 1);
        } else {
            run = 0;
        }
    }
    assert!(worst >= 1 && worst <= pr.profile.blank_run_inside, "blank-page runs of {worst}");
    assert!(run <= pr.profile.blank_run_end);
    // Folios: roman in the front matter from i, arabic from 1 at the first body page, one a page.
    let first_body = pr.book.sections.iter().position(|s| !s.front).unwrap();
    let body_start = map[first_body].page;
    let folios: Vec<String> = (0..t.page_count()).map(|i| t.folio(i)).collect();
    assert_eq!(wordsmith_book_typeset::roman_lower(14), "xiv");
    for (i, f) in folios[..body_start - 1].iter().enumerate() {
        assert_eq!(f, &wordsmith_book_typeset::roman_lower(i as u32 + 1), "front page {}", i + 1);
    }
    for (i, f) in folios[body_start - 1..].iter().enumerate() {
        assert_eq!(f, &(i + 1).to_string());
    }
    // A printed folio matches the page's folio (the foot on a body page that has one).
    let body_page = body_start + 2;
    assert_eq!(bands[body_page - 1].1, folios[body_page - 1]);
}

#[test]
fn running_heads_show_author_and_title_or_title_and_chapter_or_nothing() {
    let classic = gulliver_print("classic", "6x9");
    let literary = gulliver_print("literary", "6x9");
    let first_ch = |pr: &build::Print| {
        let i = pr.book.sections.iter().position(|s| s.kind == Kind::Chapter).unwrap();
        pr.typeset.page_map()[i].page
    };
    let c = bands(&classic.typeset);
    let p = first_ch(&classic);
    // The page after the opening is a left-hand page: the author; the next a right-hand one: the title.
    assert_eq!(c[p].0.to_lowercase(), "jonathan swift");
    assert_eq!(c[p + 1].0.to_lowercase(), "gulliver’s travels");
    let l = bands(&literary.typeset);
    let p = first_ch(&literary);
    let folio = literary.typeset.folio(p);
    assert_eq!(l[p].0, format!("{folio}Gulliver’s Travels"), "left: folio, then the title");
    let ch = &literary.book.sections.iter().find(|s| s.kind == Kind::Chapter).unwrap().head;
    assert!(l[p + 1].0.starts_with(ch.as_str()) || ch.is_empty(), "right: the chapter, then the folio: {:?}", l[p + 1].0);
    // A design without running heads: none anywhere, folios at the foot still.
    let mut d = builtin::get("classic").unwrap();
    d.heads.style = wordsmith_book_design::design::Heads::None;
    d.id = "no-heads".into();
    let mut proj = project();
    proj.design.id = "no-heads".into();
    let own = std::collections::BTreeMap::from([("no-heads".to_owned(), wordsmith_book_design::design::write(&d).unwrap())]);
    let pr = build::print(&proj, &own, &gulliver(), &Edition::paperback(), &|_| None).unwrap();
    assert!(bands(&pr.typeset).iter().all(|(h, _, _)| h.is_empty()));
}

#[test]
fn the_layout_settles_and_gives_the_same_bytes_again() {
    let a = gulliver_print("fantasy", "5.5x8.5");
    let b = gulliver_print("fantasy", "5.5x8.5");
    assert!(a.typeset.passes() <= MAX_PASSES);
    assert_eq!(a.typeset.page_count(), b.typeset.page_count());
    assert!(a.typeset.pdf().unwrap() == b.typeset.pdf().unwrap());
}
