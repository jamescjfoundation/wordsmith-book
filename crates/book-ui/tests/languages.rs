// SPDX-License-Identifier: MIT OR Apache-2.0
//! Interface languages (plan R2.16, ACC-3, on the core's catalogs, C4): every interface text is
//! in every catalog and no catalog keeps a text nothing uses; with Spanish on, Book's texts and
//! the shell's show in Spanish, and English comes back unchanged. (Its own test binary: the
//! language is process-wide.)

use std::path::Path;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::{Shell, egui};
use serde_json::json;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, languages, shell_options};

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// The interface's sources: Book's interface and app, and the names the model and the cleanup
/// give for the interface to show.
fn sources() -> String {
    fn walk(dir: &Path, out: &mut String) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let text = std::fs::read_to_string(&p).unwrap();
                out.push_str(text.split("#[cfg(test)]").next().unwrap_or(""));
            }
        }
    }
    let mut out = String::new();
    walk(&root().join("crates/book-ui/src"), &mut out);
    walk(&root().join("apps/book/src"), &mut out);
    for f in ["crates/book-model/src/lib.rs", "crates/book-import/src/cleanup.rs"] {
        out.push_str(std::fs::read_to_string(root().join(f)).unwrap().split("#[cfg(test)]").next().unwrap_or(""));
    }
    out
}

#[test]
fn every_interface_text_is_in_every_catalog_and_none_is_unused() {
    let src = sources();
    let used = jcj_i18n::texts_in(&src);
    assert!(used.len() > 150, "{} texts", used.len());
    let singulars = jcj_i18n::singulars_in(&src);
    assert_eq!(languages::CATALOGS.len(), languages::LANGUAGES.len() - 1, "every language but English has a catalog");
    for (code, text) in languages::CATALOGS {
        let cat = jcj_i18n::Catalog::parse(text).unwrap_or_else(|e| panic!("{code}: {e}"));
        let (missing, unused) = jcj_i18n::compare(&cat, &used, &singulars);
        assert!(missing.is_empty(), "{code} lacks:\n{}", missing.join("\n"));
        assert!(unused.is_empty(), "{code} keeps texts nothing uses:\n{}", unused.join("\n"));
        // Placeholders survive translation.
        for key in cat.keys() {
            let want: Vec<&str> = key.match_indices('{').map(|(i, _)| &key[i..=key[i..].find('}').map_or(i, |j| i + j)]).collect();
            if let Some(jcj_i18n::Entry::Text(t)) = cat.get(key) {
                for w in &want {
                    assert!(t.contains(w), "{code}: “{t}” lacks {w} of “{key}”");
                }
            }
        }
    }
}

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_window_shows_the_language_chosen_and_english_again() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("languages");
    let _ = std::fs::remove_dir_all(&dir);
    let mut app = BookApp::new();
    app.command(commands::NEW, &json!({ "path": dir.join("Own.wsbk").display().to_string(), "date": "2026-11-24" })).unwrap();
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(2);
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": "details" })));
    h.run_steps(2);
    assert!(h.query_all_by_label("Line breaks").next().is_some());
    assert_eq!(languages::set("es"), "es");
    h.run_steps(2);
    assert!(h.query_all_by_label("Line breaks").next().is_none());
    assert!(h.query_all_by_label("Saltos de línea").next().is_some(), "Book's text");
    assert!(h.query_all_by_label("Detalles").next().is_some(), "the panel's title");
    assert!(h.query_all_by_label("Archivo").next().is_some(), "the shell's text");
    // Preferences keep the language; a missing key follows the computer.
    let p: jcj_shell::prefs::Prefs<wordsmith_book_ui::prefs::BookPrefs> = serde_json::from_str(r#"{"app": {"language": "es"}}"#).unwrap();
    assert_eq!(p.app.language, "es");
    let p: jcj_shell::prefs::Prefs<wordsmith_book_ui::prefs::BookPrefs> = serde_json::from_str(r#"{"app": {}}"#).unwrap();
    assert_eq!(p.app.language, "");
    assert_eq!(languages::set("en"), "en");
    h.run_steps(2);
    assert!(h.query_all_by_label("Line breaks").next().is_some());
    assert!(h.query_all_by_label("Details").next().is_some());
}
