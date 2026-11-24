// SPDX-License-Identifier: MIT OR Apache-2.0
//! Writes sample pages of the test book in three designs under `target/look/` for a person to
//! look at (`cargo test -p wordsmith-book-check --test pages_for_review -- --ignored`).

mod common;
use common::*;
#[test]
#[ignore = "writes files for a person to look at"]
fn pages() {
    for id in ["classic", "fantasy", "thriller"] {
        let (_, _, t) = print(&gulliver(), &wordsmith_book_design::builtin::get(id).unwrap());
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/look");
        std::fs::create_dir_all(&dir).unwrap();
        let map = t.page_map();
        let (pb, _, _) = print(&gulliver(), &wordsmith_book_design::builtin::get(id).unwrap());
        let ch = pb.sections.iter().position(|s| s.kind == wordsmith_book_typeset::Kind::Chapter).unwrap();
        let part = pb.sections.iter().position(|s| s.kind == wordsmith_book_typeset::Kind::Part).unwrap();
        let contents = pb.sections.iter().position(|s| s.title == "Contents").unwrap();
        for (i, p) in [map[contents].page - 1, map[part].page - 1, map[ch].page - 1, map[ch].page, map[ch].page + 1, 3].iter().enumerate() {
            let (_, _, png) = t.page_png(*p, 90.0).unwrap();
            std::fs::write(dir.join(format!("{id}-{i}.png")), png).unwrap();
        }
        println!("{id}: {} pages, {} passes", t.page_count(), t.passes());
    }
}
