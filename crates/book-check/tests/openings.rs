// SPDX-License-Identifier: MIT OR Apache-2.0
//! Openings (plan M5.3): each design's chapter opening on the test book (the first line's place,
//! a drop cap's depth), small-capital lead-ins, scene breaks, part pages, and no page of the test
//! books that ends or begins with one line of a paragraph.

mod common;

use common::*;
use wordsmith_book_check::build;
use wordsmith_book_design::builtin;
use wordsmith_book_design::design::FirstLine;
use wordsmith_book_model::Edition;
use wordsmith_book_typeset::Kind;

fn print_with(design: &str, book: &wordsmith_manuscript_format::book::Book) -> build::Print {
    let mut p = project();
    p.design.id = design.into();
    build::print(&p, &Default::default(), book, &Edition::paperback(), &|_| None).unwrap()
}

/// The glyphs of page `index`: (text, x, y from the top).
fn glyphs(pr: &build::Print, pdf: &[u8], index: usize) -> Vec<(String, f64, f64)> {
    let pages = jcj_output::read_pages(pdf, &jcj_output::ReadLimits { max_input_bytes: 256 << 20, max_pages: 10_000, max_glyphs_per_page: 20_000 }).unwrap();
    let h = f64::from(pr.typeset.page_size(index).unwrap().1);
    pages[index].glyphs.iter().filter_map(|g| g.text.clone().filter(|t| !t.trim().is_empty()).map(|t| (t, g.x, h - g.y))).collect()
}

#[test]
fn every_designs_opening_sinks_and_drops_its_capital() {
    let book = gulliver();
    for d in builtin::all() {
        let pr = print_with(&d.id, &book);
        let pdf = pr.typeset.pdf().unwrap();
        let i = pr.book.sections.iter().position(|s| s.kind == Kind::Chapter).unwrap();
        let page = pr.typeset.page_map()[i].page - 1;
        let body = jcj_layout::page_boxes(pr.typeset.layout())[page].body;
        let g = glyphs(&pr, &pdf, page);
        // The chapter's text starts with "The author gives some account".
        let letters: Vec<String> = g.iter().map(|(t, _, _)| t.to_lowercase()).collect();
        let first = (0..g.len()).find(|&k| letters[k].chars().all(char::is_alphabetic) && letters[k..].iter().filter(|t| t.chars().all(char::is_alphabetic)).take(9).cloned().collect::<String>() == "theauthor").unwrap_or_else(|| panic!("{}: no first words", d.id));
        let first_y = g[first].2;
        let sink = f64::from(body.y) + f64::from(d.chapter.sink * body.h);
        assert!(first_y > sink, "{}: the text starts at {first_y:.0}, below the sink at {sink:.0}", d.id);
        let next = g[first + 1..].iter().find(|(t, _, _)| t.chars().all(char::is_alphabetic)).unwrap();
        let line = f64::from(d.body.size * d.body.leading);
        match d.chapter.first_line {
            FirstLine::DropCap => {
                // The dropped capital's foot sits (lines - 1) lines below the first line's.
                let depth = (g[first].2 - next.2) / line;
                assert!((depth - f64::from(d.chapter.drop_lines - 1)).abs() < 0.6, "{}: the drop cap goes down {depth:.2} lines", d.id);
            }
            FirstLine::SmallCaps => {
                // "THE" in small capitals: the lead-in reads back in capitals.
                assert!(g[first + 1..first + 3].iter().all(|(t, _, _)| t.chars().all(char::is_uppercase)), "{}: small-capital lead-in", d.id);
            }
            FirstLine::None => assert!((g[first].2 - next.2).abs() < 1.0, "{}", d.id),
        }
    }
}

#[test]
fn scene_breaks_and_part_pages_follow_the_design() {
    let mut book = gulliver();
    // Put a scene break in the first chapter.
    let ch = &mut book.body[0].children[0];
    ch.content.insert(1, wordsmith_manuscript_model::Block::SceneBreak);
    for (design, mark) in [("classic", Some("❦")), ("literary", None)] {
        let pr = print_with(design, &book);
        let paras = pr.typeset.printed_paragraphs();
        let at = paras.iter().position(|(t, _)| t.starts_with("My father had")).unwrap();
        match mark {
            Some(m) => assert_eq!(paras[at - 1].0, m, "{design}: the mark between the scenes"),
            None => assert_eq!(paras[at - 1].0, "", "{design}: a blank line between the scenes"),
        }
        let doc = pr.typeset.document();
        let p = jcj_doc::walk::body_paragraphs(doc).into_iter().find(|(_, p)| p.text.starts_with("My father")).unwrap().1.clone();
        // After a break: no indent; in the literary design, a small-capital start.
        assert_eq!(p.props.indent_first, Some(0.0), "{design}");
        assert_eq!(p.runs.first().and_then(|r| r.props.small_caps) == Some(true), mark.is_none(), "{design}");
    }
    // A part page shows its number and title only, on a right-hand page, without a head.
    let pr = print_with("classic", &gulliver());
    let i = pr.book.sections.iter().position(|s| s.kind == Kind::Part).unwrap();
    let start = &pr.typeset.page_map()[i];
    assert_eq!(start.page % 2, 1);
    let text: String = glyphs(&pr, &pr.typeset.pdf().unwrap(), start.page - 1).iter().map(|g| g.0.clone()).collect();
    assert!(text.to_lowercase().starts_with("partone") && text.contains("VOYAGETOLILLIPUT"), "{text}");
}

#[test]
fn no_page_ends_or_begins_with_one_line_of_a_paragraph() {
    for (name, book) in [("Gulliver's Travels", gulliver()), ("Waverley", waverley())] {
        let pr = print_with("classic", &book);
        let pieces = jcj_layout::pages::body_pieces(pr.typeset.layout());
        let mut checked = 0;
        for (page, ps) in pieces.iter().enumerate() {
            for piece in ps {
                let Some(lines) = &piece.lines else { continue };
                let n = lines.end - lines.start;
                if piece.continues {
                    assert!(n >= 2, "{name}: page {} ends with one line of a paragraph", page + 1);
                    checked += 1;
                }
                if piece.continued {
                    assert!(n >= 2, "{name}: page {} begins with one line of a paragraph", page + 1);
                }
            }
        }
        assert!(checked > 100, "{name}: {checked} paragraphs split across pages");
    }
}
