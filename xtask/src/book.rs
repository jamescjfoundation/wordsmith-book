// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book's own tasks: `app-layers` (the table in `AGENTS.md`, "Where each crate's code
//! goes"), `assets` and `fixtures`. This module is the app's own; the core never syncs it.

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

/// Outside crates and the lowest layer allowed to use each. The table in `AGENTS.md` ("Where each
/// crate's code goes") says the same in words; change both together.
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
    ("jcj-i18n", 5),
    ("jcj-theme", 5),
    ("jcj-agents", 5),
    ("eframe", 5),
    ("egui_kittest", 5),
];

/// Crates only one package may use: an embedded typesetting engine would stay inside
/// `book-typeset`. None of these is in the graph today: print is on the core's layout (decision
/// 0003). The rule holds if the engine is ever added.
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
    fn layers_point_down() {
        assert!(layer_problem("wordsmith-book-format", "wordsmith-book-model").is_none());
        assert!(layer_problem("wordsmith-book-model", "wordsmith-book-format").is_some());
        assert!(layer_problem("wordsmith-book-format", "wordsmith-book-design").is_some());
        assert!(layer_problem("wordsmith-book-model", "jcj-shell").is_some());
        assert!(layer_problem("wordsmith-book-ui", "jcj-shell").is_none());
        assert!(layer_problem("wordsmith-book-ui", "jcj-i18n").is_none());
        assert!(layer_problem("wordsmith-book-check", "jcj-i18n").is_some());
        assert!(layer_problem("wordsmith-book-epub", "typst").is_some());
        assert!(layer_problem("wordsmith-book-typeset", "typst").is_none());
    }

    #[test]
    fn assets_listed_by_path_or_folder() {
        let text = "| `assets/fonts/` | x |\n| `assets/brand/tokens.json` | y |";
        assert!(listed(text, "assets/fonts/serif/a.otf"));
        assert!(listed(text, "assets/brand/tokens.json"));
        assert!(!listed(text, "assets/brand/mark.svg"));
    }
}
