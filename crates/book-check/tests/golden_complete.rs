// SPDX-License-Identifier: MIT OR Apache-2.0
//! The golden files (plan R4.6): each folder holds exactly the files its test writes, one for
//! every book and design it covers, so a comparison never passes over a missing file and no
//! stale file is left behind. The comparisons themselves are the print, EPUB and v0.1 tests.

use std::collections::BTreeSet;
use std::path::Path;

use wordsmith_book_design::builtin;

fn listed(dir: &str) -> BTreeSet<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden").join(dir);
    std::fs::read_dir(path).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect()
}

#[test]
fn every_golden_file_is_one_a_test_compares() {
    let designs: Vec<String> = builtin::FILES.iter().map(|(id, _)| (*id).to_owned()).collect();
    let mut print = BTreeSet::new();
    let mut epub = BTreeSet::new();
    for d in &designs {
        print.insert(format!("gulliver-{d}-pod-paperback.txt"));
        print.insert(format!("waverley-{d}-pod-paperback.txt"));
        print.insert(format!("gulliver-{d}-generic-pdf.txt"));
        epub.insert(format!("gulliver-{d}.txt"));
        epub.insert(format!("waverley-{d}.txt"));
    }
    assert_eq!(listed("print"), print);
    assert_eq!(listed("epub"), epub);
    // The test of v0.1 stays on the six designs 0.1 shipped.
    let mut v01 = BTreeSet::new();
    for source in ["gulliver-docx", "gulliver-manuscript", "waverley-docx"] {
        for d in ["classic", "literary", "romance", "thriller", "fantasy", "non-fiction"] {
            v01.insert(format!("{source}-{d}.txt"));
        }
    }
    assert_eq!(listed("v0.1"), v01);
}
