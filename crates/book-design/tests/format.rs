// SPDX-License-Identifier: MIT OR Apache-2.0
//! The design format (plan M4.1): round trips, every example in `docs/design-format.md`, hostile
//! files, and each validation message.

use std::path::Path;

use wordsmith_book_design::design::{self, Error, FirstLine, Hyphenation};
use wordsmith_book_design::{Design, known_role};

fn read(text: &str) -> Result<Design, Error> {
    design::read(text, &known_role)
}

#[test]
fn designs_round_trip_exactly() {
    let mut d = Design { id: "mine".into(), name: "Mine".into(), genres: vec!["fantasy".into()], ..Design::default() };
    d.body.size = 10.5;
    d.body.hyphenation = Hyphenation::Strict;
    d.chapter.first_line = FirstLine::DropCap;
    d.chapter.ornament = "❦".into();
    let text = design::write(&d).unwrap();
    assert_eq!(read(&text).unwrap(), d);
    assert_eq!(design::write(&read(&text).unwrap()).unwrap(), text);
}

#[test]
fn every_example_in_the_design_format_doc_reads() {
    let doc = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design-format.md")).unwrap();
    let mut n = 0;
    for part in doc.split("<!-- example: ").skip(1) {
        let body = part.split("```").nth(1).unwrap().split_once('\n').unwrap().1;
        let d = read(body).unwrap_or_else(|e| panic!("{}: {e}", part.lines().next().unwrap()));
        assert_eq!(d.schema, 1);
        n += 1;
    }
    assert_eq!(n, 3);
    // The version-0 example came across with its size.
    let v0 = doc.split("<!-- example: version 0").nth(1).unwrap().split("```").nth(1).unwrap().split_once('\n').unwrap().1;
    assert_eq!(read(v0).unwrap().body.size, 10.5);
}

#[test]
fn hostile_and_damaged_files_are_refused_cleanly() {
    assert_eq!(read(&format!("schema = 1\nid = \"x\"\nname = \"X\"\n# {}", "x".repeat(design::MAX_FILE))), Err(Error::TooLarge));
    assert!(matches!(read("schema = [[["), Err(Error::NotADesign(_))));
    assert!(matches!(read("id = \"x\"\nname = \"X\"\n"), Err(Error::NotADesign(_))));
    assert!(matches!(read("schema = -1\nid = \"x\"\nname = \"X\"\n"), Err(Error::NotADesign(_))));
    assert_eq!(read("schema = 7\nid = \"x\"\nname = \"X\"\n"), Err(Error::Newer(7)));
    assert!(matches!(read("schema = 1\nid = \"x\"\nname = \"X\"\n[body]\nsiz = 10\n"), Err(Error::NotADesign(_))), "a misspelt key is refused");
    assert!(matches!(read("schema = 1\nid = \"x\"\nname = \"X\"\n[body]\nsize = \"big\"\n"), Err(Error::NotADesign(_))));
    assert!(matches!(read("schema = 1\nid = \"x\"\nname = \"X\"\n[chapter]\nstart = \"sideways\"\n"), Err(Error::NotADesign(_))));
    assert!(Error::Newer(7).to_string().contains("newer version"));
}

/// The one problem a design with `change` applied has.
fn problem(change: impl FnOnce(&mut Design)) -> String {
    let mut d = Design { id: "x".into(), name: "X".into(), ..Design::default() };
    change(&mut d);
    let p = design::problems(&d, &known_role);
    assert_eq!(p.len(), 1, "{p:?}");
    p[0].clone()
}

#[test]
fn each_validation_message_says_what_is_wrong() {
    assert!(problem(|d| d.id = "bad id!".into()).contains("id must be letters"));
    assert!(problem(|d| d.name = " ".into()).contains("needs a name"));
    assert!(problem(|d| d.body.font = "comic".into()).contains("not one of Book's font roles"));
    assert!(problem(|d| d.chapter.font = "comic".into()).contains("chapter font"));
    assert_eq!(problem(|d| d.body.size = 30.0), "The body size must be between 7 and 24 points; it is 30.");
    assert!(problem(|d| d.body.size = 6.0).contains("body size"));
    assert!(problem(|d| d.body.leading = 3.0).contains("leading"));
    assert!(problem(|d| d.body.indent = 9.0).contains("first-line indent"));
    assert!(problem(|d| d.body.orphans = 0).contains("orphans"));
    assert!(problem(|d| d.body.widows = 9).contains("widows"));
    assert!(problem(|d| d.chapter.sink = 0.9).contains("chapter sink"));
    assert!(problem(|d| d.chapter.number_size = 100.0).contains("chapter number size"));
    assert!(problem(|d| d.chapter.title_size = 1.0).contains("chapter title size"));
    assert!(problem(|d| {
        d.chapter.first_line = FirstLine::DropCap;
        d.chapter.drop_lines = 7;
    })
    .contains("drop cap"));
    assert!(problem(|d| d.chapter.ornament = "too many glyphs here".into()).contains("ornament"));
    assert!(problem(|d| d.scene_break.mark = " ".into()).contains("needs a mark"));
    assert!(problem(|d| d.scene_break.mark = "end".into()).contains("no letters"));
    assert!(problem(|d| d.part.title_size = 200.0).contains("part title size"));
    assert!(problem(|d| d.part.number_size = 1.0).contains("part number size"));
    assert!(problem(|d| d.part.sink = 0.8).contains("part sink"));
    assert!(problem(|d| d.heads.size = 30.0).contains("running head size"));
    assert!(problem(|d| d.ebook.indent = 9.0).contains("ebook indent"));
    assert!(problem(|d| d.body.size = f32::NAN).contains("body size"));
}
