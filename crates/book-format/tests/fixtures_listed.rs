// SPDX-License-Identifier: MIT OR Apache-2.0
//! Every fixture file is listed in `docs/fixtures.md`, and every listed file exists (plan M0.5).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn walk(dir: &Path, base: &Path, out: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            // A Manuscript or Book project folder is one fixture: list the folder.
            let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if name.ends_with(".wsm") || name.ends_with(".wsbk") {
                out.insert(format!("{}/", p.strip_prefix(base).unwrap_or(&p).display()));
            } else {
                walk(&p, base, out);
            }
        } else {
            out.insert(p.strip_prefix(base).unwrap_or(&p).display().to_string());
        }
    }
}

#[test]
fn every_fixture_is_listed_and_every_listed_fixture_exists() {
    let root = root();
    let doc = std::fs::read_to_string(root.join("docs/fixtures.md")).expect("docs/fixtures.md");
    let listed: BTreeSet<String> = doc
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|s| s.starts_with("tests/fixtures/") && !s.ends_with("/*") && s.contains('.'))
        .map(str::to_owned)
        .collect();
    let mut present = BTreeSet::new();
    walk(&root.join("tests/fixtures"), &root, &mut present);
    let missing: Vec<_> = listed.difference(&present).collect();
    let unlisted: Vec<_> = present.difference(&listed).collect();
    assert!(missing.is_empty(), "listed in docs/fixtures.md but missing: {missing:?}");
    assert!(unlisted.is_empty(), "fixtures not listed in docs/fixtures.md: {unlisted:?}");
    assert!(listed.len() >= 13, "the novels and their documents are listed");
}
