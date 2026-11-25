// SPDX-License-Identifier: MIT OR Apache-2.0
//! Screen readers (plan R2.17, ACC-2): every control in every view a screen reader can reach has
//! a name. The views are walked through the egui harness's accessibility tree (AccessKit, what
//! screen readers read): each panel with a book open, the File page's pages, the dialogs and the
//! import review; a button, check box, text field, number field, link, tab or picture without a
//! name fails the test.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use egui_kittest::Harness;
use egui_kittest::kittest::NodeT;
use jcj_shell::egui::accesskit::Role;
use jcj_shell::{Shell, egui};
use serde_json::json;
use wordsmith_book_ui::outline::Key;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

fn tree(dir: &Path, base: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            tree(&p, base, out);
        } else {
            out.insert(p.strip_prefix(base).unwrap().to_owned(), std::fs::read(&p).unwrap());
        }
    }
}

fn linked_app() -> BookApp {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("access");
    let _ = std::fs::remove_dir_all(&dir);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/gullivers-travels.wsm");
    let mut files = BTreeMap::new();
    tree(&src, &src, &mut files);
    for (rel, bytes) in files {
        let p = dir.join("Gulliver.wsm").join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    let mut app = BookApp::new();
    app.command(
        commands::NEW,
        &json!({ "path": dir.join("Gulliver.wsbk").display().to_string(), "manuscript": "../Gulliver.wsm", "date": "2026-11-24" }),
    )
    .unwrap();
    app
}

struct St {
    shell: Shell,
    app: BookApp,
}

/// The roles a screen reader announces as something to act on or look at.
const NAMED: [Role; 14] = [
    Role::Button,
    Role::CheckBox,
    Role::RadioButton,
    Role::Switch,
    Role::TextInput,
    Role::MultilineTextInput,
    Role::SpinButton,
    Role::Slider,
    Role::ComboBox,
    Role::Link,
    Role::Tab,
    Role::MenuItem,
    Role::Image,
    Role::Splitter,
];

/// How many controls the walk has looked at.
static SEEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The controls in the window now that have no name: role and where (the nearest named
/// ancestor).
fn unnamed(h: &Harness<'_, St>, view: &str) -> Vec<String> {
    let mut out = Vec::new();
    for n in h.root().children_recursive() {
        let a = n.accesskit_node();
        if a.is_hidden() || !NAMED.contains(&a.role()) {
            continue;
        }
        SEEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let name = a.label().unwrap_or_default();
        if name.trim().is_empty() && a.value().is_none_or(|v| v.trim().is_empty() || a.role() != Role::Button) {
            let mut near = String::new();
            let mut p = n.parent();
            while let Some(x) = p {
                if let Some(l) = x.accesskit_node().label().filter(|l| !l.trim().is_empty()) {
                    near = l;
                    break;
                }
                p = x.parent();
            }
            out.push(format!("{view}: a {:?} with no name (near “{near}”)", a.role()));
        }
    }
    out
}

fn panel(h: &mut Harness<'_, St>, id: &str) {
    h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::PANEL, json!({ "panel": id })));
    h.run_steps(3);
}

#[test]
fn every_control_in_every_view_has_a_name() {
    let app = linked_app();
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h =
        Harness::builder().with_size(egui::vec2(1600.0, 1100.0)).build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app });
    h.run_steps(3);
    let mut problems = unnamed(&h, "the window");
    for id in ["outline", "cleanup", "details", "designs", "preview", "ebook", "preflight"] {
        panel(&mut h, id);
        // The previews lay out in the background: give them time to show pages.
        for _ in 0..40 {
            h.step();
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        h.run_steps(2);
        problems.extend(unnamed(&h, id));
        panel(&mut h, id);
    }
    // A chapter selected: its text in the middle, with each paragraph's Fix button; then one of
    // Book's elements.
    let rows = {
        let st = h.state();
        let b = st.app.book.as_ref().unwrap();
        wordsmith_book_ui::outline::rows(&b.files.project, b.source.as_ref().ok().map(|s| &s.book))
    };
    for key in [
        rows.iter().find(|r| matches!(r.key, Key::Section(_))).map(|r| r.key.clone()),
        rows.iter().find(|r| matches!(r.key, Key::Element(_))).map(|r| r.key.clone()),
    ] {
        h.state_mut().app.outline.selected = Some(key.unwrap());
        h.run_steps(3);
        problems.extend(unnamed(&h, "the text view"));
    }
    for page in ["new", "open", "save", "export", "preferences", "keys", "about"] {
        h.state_mut().shell.run(jcj_shell::Cmd::with(jcj_shell::commands::shell_ids::FILE_PAGE, json!({ "page": page })));
        h.run_steps(3);
        problems.extend(unnamed(&h, &format!("the File page's {page} page")));
    }
    h.state_mut().shell.run(jcj_shell::Cmd::new(jcj_shell::commands::shell_ids::CLOSE_FILE_PAGE));
    h.run_steps(2);
    // The dialogs.
    h.state_mut().app.open_dialog = Some(String::new());
    h.run_steps(3);
    problems.extend(unnamed(&h, "the open dialog"));
    h.state_mut().app.open_dialog = None;
    h.state_mut().shell.run(jcj_shell::Cmd::new(jcj_shell::commands::shell_ids::PALETTE));
    h.run_steps(3);
    problems.extend(unnamed(&h, "the command palette"));
    h.state_mut().shell.run(jcj_shell::Cmd::new(jcj_shell::commands::shell_ids::PALETTE));
    h.run_steps(2);
    // The import review.
    let docx = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels/waverley.docx");
    h.state_mut().app.command(commands::IMPORT_REVIEW, &json!({ "file": docx.display().to_string() })).unwrap();
    h.run_steps(3);
    problems.extend(unnamed(&h, "the import review"));
    problems.sort();
    problems.dedup();
    let seen = SEEN.load(std::sync::atomic::Ordering::Relaxed);
    assert!(seen > 300, "the walk looked at {seen} controls");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
