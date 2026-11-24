// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book's own tasks: `app-layers` (plan section 5.2), `assets` and `names` (plan
//! section 4, rules 2 and 5). This module is the app's own; the core never syncs it.

use std::collections::BTreeSet;
use std::path::Path;

use crate::util::{TaskResult, direct_deps, git_output, report, workspace_members};

/// Each workspace package's layer. A package may depend on packages of a lower layer only.
pub const LAYERS: &[(&str, u8)] = &[
    ("wordsmith-book-model", 1),
    ("wordsmith-book-format", 2),
    ("wordsmith-book-design", 2),
    ("wordsmith-book-import", 3),
    ("wordsmith-book-typeset", 3),
    ("wordsmith-book-epub", 3),
    ("wordsmith-book-check", 4),
    ("wordsmith-book-ui", 5),
    ("wordsmith-book", 6),
    ("wordsmith-book-cli", 6),
];

/// Outside crates and the lowest layer allowed to use each (plan section 5.2).
pub const FIRST_LAYER: &[(&str, u8)] = &[
    ("wordsmith-manuscript-model", 1),
    ("jcj-doc", 1),
    ("wordsmith-manuscript-format", 2),
    ("jcj-project", 2),
    ("jcj-interchange", 3),
    ("jcj-layout", 3),
    ("jcj-output", 3),
    ("jcj-proof", 3),
    ("jcj-features", 3),
    ("jcj-engine", 5),
    ("jcj-shell", 5),
    ("jcj-theme", 5),
    ("jcj-agents", 5),
    ("eframe", 5),
    ("egui_kittest", 5),
];

/// Crates only one package may use: the typesetting engine stays inside `book-typeset`
/// (plan section 5.6).
pub const ONLY_IN: &[(&str, &str)] =
    &[("typst", "wordsmith-book-typeset"), ("typst-pdf", "wordsmith-book-typeset"), ("typst-render", "wordsmith-book-typeset")];

fn layer_of(name: &str) -> Option<u8> {
    LAYERS.iter().find(|(n, _)| *n == name).map(|(_, l)| *l)
}

/// Why one dependency breaks the layer table, if it does.
pub fn layer_problem(member: &str, dep: &str) -> Option<String> {
    let own = layer_of(member)?;
    if let Some(theirs) = layer_of(dep) {
        if theirs >= own {
            return Some(format!("{member} (layer {own}) depends on {dep} (layer {theirs}); layers point down"));
        }
        return None;
    }
    if let Some((_, first)) = FIRST_LAYER.iter().find(|(n, _)| *n == dep)
        && own < *first
    {
        return Some(format!("{member} (layer {own}) uses {dep}, allowed from layer {first}"));
    }
    if let Some((_, only)) = ONLY_IN.iter().find(|(n, _)| *n == dep)
        && member != *only
    {
        return Some(format!("{member} uses {dep}, which only {only} may use"));
    }
    None
}

/// `cargo xtask app-layers`: every package is in the table and depends only downwards.
pub fn app_layers(root: &Path) -> TaskResult {
    let mut failures = Vec::new();
    for m in workspace_members(root)? {
        if m.name == "xtask" {
            continue;
        }
        if layer_of(&m.name).is_none() {
            failures.push(format!("{} has no layer in xtask/src/book.rs", m.name));
            continue;
        }
        for dep in direct_deps(root, &m.name)? {
            if let Some(p) = layer_problem(&m.name, &dep) {
                failures.push(p);
            }
        }
    }
    report(failures, "every package depends only on the layers under it")
}

/// Files every release carries.
const LEGAL: &[&str] = &["LICENSE-MIT", "LICENSE-APACHE", "NOTICE", "ATTRIBUTION.md", "SECURITY.md"];

/// `cargo xtask assets`: every tracked file under `assets/` is named in ATTRIBUTION.md, by its
/// path or a folder above it, and the legal files exist.
pub fn assets(root: &Path) -> TaskResult {
    let mut failures = Vec::new();
    for f in LEGAL {
        if !root.join(f).is_file() {
            failures.push(format!("{f} is missing"));
        }
    }
    let attribution = std::fs::read_to_string(root.join("ATTRIBUTION.md")).unwrap_or_default();
    let tracked = git_output(root, &["ls-files", "assets"])?;
    for file in tracked.lines().filter(|l| !l.is_empty()) {
        if !listed(&attribution, file) {
            failures.push(format!("{file} is not listed in ATTRIBUTION.md (by path or folder)"));
        }
    }
    report(failures, "every asset is listed in ATTRIBUTION.md and the legal files exist")
}

/// Whether `file` or one of its folders appears in the attribution text in backticks.
pub fn listed(attribution: &str, file: &str) -> bool {
    let mut path = file;
    loop {
        if attribution.contains(&format!("`{path}`")) || attribution.contains(&format!("`{path}/`")) {
            return true;
        }
        match path.rfind('/') {
            Some(i) => path = &path[..i],
            None => return false,
        }
    }
}

/// Paths exempt from `names`: the fonts' own files, the public-domain texts and the print golden
/// summaries (which quote those texts) name what they
/// name (plan section 12).
pub const NAMES_EXEMPT: &[&str] = &["ATTRIBUTION.md", "assets/fonts/", "tests/fixtures/novels/", "tests/golden/print/"];

/// The largest file `names` reads; larger files are data.
const MAX_NAMES_SCAN: u64 = 16 * 1024 * 1024;

/// The names list: one word or phrase per line, `#` starts a comment.
pub fn parse_names(text: &str) -> Vec<String> {
    text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(str::to_lowercase).collect()
}

/// The names that occur in `text` as whole words, case ignored.
pub fn names_in(text: &str, names: &[String]) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut out = BTreeSet::new();
    for name in names {
        let mut from = 0;
        while let Some(i) = lower[from..].find(name.as_str()) {
            let start = from + i;
            let end = start + name.len();
            let before = lower[..start].chars().next_back();
            let after = lower[end..].chars().next();
            if !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric) {
                out.insert(name.clone());
                break;
            }
            from = start + lower[start..].chars().next().map_or(1, char::len_utf8);
        }
    }
    out.into_iter().collect()
}

/// `cargo xtask names`: no tracked file and no commit message on this branch names a company,
/// store, service or bundled font family from `.private/names.txt`.
pub fn names(root: &Path) -> TaskResult {
    let list = match std::fs::read_to_string(root.join(".private/names.txt")) {
        Ok(t) => parse_names(&t),
        Err(_) => {
            println!("note: .private/names.txt not found; nothing to check");
            return Ok(());
        }
    };
    let mut failures = Vec::new();
    let tracked = git_output(root, &["ls-files"])?;
    for file in tracked.lines().filter(|l| !l.is_empty()) {
        if NAMES_EXEMPT.iter().any(|e| file == *e || (e.ends_with('/') && file.starts_with(e))) {
            continue;
        }
        let path = root.join(file);
        if std::fs::metadata(&path).map(|m| m.len() > MAX_NAMES_SCAN).unwrap_or(true) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        for (n, line) in text.lines().enumerate() {
            for name in names_in(line, &list) {
                failures.push(format!("{file}:{}: names `{name}`", n + 1));
            }
        }
    }
    let log = git_output(root, &["log", "--format=%H%n%B%n--end--", "HEAD"]).unwrap_or_default();
    let mut commit = String::new();
    for line in log.lines() {
        if commit.is_empty() {
            commit = line.chars().take(10).collect();
            continue;
        }
        if line == "--end--" {
            commit.clear();
            continue;
        }
        for name in names_in(line, &list) {
            failures.push(format!("commit {commit}: message names `{name}`"));
        }
    }
    report(failures, "no listed name in tracked files or commit messages")
}

/// The Manuscript project fixture: its source text and the folder written from it.
pub const MANUSCRIPT_FIXTURE: (&str, &str) = ("tests/fixtures/novels/gullivers-travels.txt", "tests/fixtures/novels/gullivers-travels.wsm");

/// `cargo xtask fixtures`: write the fixtures that need Rust (the Manuscript project, through
/// Manuscript's public writer). The others come from `tools/make-fixtures.py`.
pub fn fixtures(root: &Path) -> TaskResult {
    let (src, out) = MANUSCRIPT_FIXTURE;
    crate::util::cargo(root, &["run", "--quiet", "-p", "wordsmith-book-format", "--example", "manuscript_fixture", "--", src, out])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_words_only_case_ignored() {
        let names = parse_names("# a comment\nAcme\nBig Store\n\n");
        assert_eq!(names, vec!["acme".to_owned(), "big store".to_owned()]);
        assert_eq!(names_in("Made at ACME today", &names), vec!["acme".to_owned()]);
        assert!(names_in("acmes and macme", &names).is_empty());
        assert_eq!(names_in("the big  store, the Big Store.", &names), vec!["big store".to_owned()]);
        assert!(names_in("", &names).is_empty());
    }

    #[test]
    fn layers_point_down() {
        assert!(layer_problem("wordsmith-book-format", "wordsmith-book-model").is_none());
        assert!(layer_problem("wordsmith-book-model", "wordsmith-book-format").is_some());
        assert!(layer_problem("wordsmith-book-format", "wordsmith-book-design").is_some());
        assert!(layer_problem("wordsmith-book-model", "jcj-shell").is_some());
        assert!(layer_problem("wordsmith-book-ui", "jcj-shell").is_none());
        assert!(layer_problem("wordsmith-book-epub", "typst").is_some());
        assert!(layer_problem("wordsmith-book-typeset", "typst").is_none());
    }

    #[test]
    fn assets_listed_by_path_or_folder() {
        let text = "| `assets/fonts/` | x |\n| `assets/brand/book/tokens.json` | y |";
        assert!(listed(text, "assets/fonts/serif/a.otf"));
        assert!(listed(text, "assets/brand/book/tokens.json"));
        assert!(!listed(text, "assets/brand/book/mark.svg"));
    }
}
