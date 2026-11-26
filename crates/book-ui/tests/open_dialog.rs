// SPDX-License-Identifier: MIT OR Apache-2.0
//! *Open…* is the system's Open dialog, asked for through the shell (`jcj_shell::files`): the
//! dialog offers a project's `book.toml` and packed projects, and the file chosen opens the
//! book. A stand-in answers in the system's place.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use egui_kittest::Harness;
use jcj_shell::commands::ids;
use jcj_shell::files::{FileAnswer, FileAsk, FileDialogs, FileKind};
use jcj_shell::{Cmd, Shell, egui};
use serde_json::json;
use wordsmith_book_ui::{BookApp, NAME, brand, commands, shell_options};

struct St {
    shell: Shell,
    files: FileDialogs,
    app: BookApp,
}

fn frame(ui: &mut egui::Ui, st: &mut St) {
    st.shell.ui(ui, &mut st.app);
    st.files.tick(&mut st.shell, ui.ctx());
}

#[test]
fn open_asks_for_a_file_and_opens_the_book_chosen() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("open-dialog");
    let _ = std::fs::remove_dir_all(&dir);
    let project = dir.join("Own.wsbk");
    let mut made = BookApp::new();
    made.command(commands::NEW, &json!({ "path": project.display().to_string(), "date": "2026-11-26" })).unwrap();

    let asked: Rc<RefCell<Vec<FileAsk>>> = Rc::default();
    let answer: Rc<RefCell<Option<PathBuf>>> = Rc::default();
    let mut files = FileDialogs::new();
    let (seen, next) = (asked.clone(), answer.clone());
    files.answer_with(Some(Box::new(move |ask| {
        seen.borrow_mut().push(ask.clone());
        next.borrow_mut().take().map_or(FileAnswer::Cancelled, |p| FileAnswer::Chosen(vec![p]))
    })));
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(frame, St { shell, files, app: BookApp::new() });
    h.run_steps(2);

    // Closing the dialog opens nothing.
    h.state_mut().shell.run(Cmd::new(ids::OPEN));
    h.run_steps(4);
    assert_eq!(asked.borrow().len(), 1, "one dialog was asked for");
    assert!(h.state().app.book.is_none());
    {
        let ask = &asked.borrow()[0];
        assert_eq!(ask.kind, FileKind::Open);
        assert_eq!(ask.title, "Open a Book");
        let extensions: Vec<&str> = ask.filters.iter().flat_map(|f| f.1.iter().map(String::as_str)).collect();
        assert_eq!(extensions, ["wsbk", "wsbkz", "toml"], "a project, a packed project, or a project's book.toml");
    }

    // Choosing the project's book.toml opens the project.
    *answer.borrow_mut() = Some(project.join("book.toml"));
    h.state_mut().shell.run(Cmd::new(ids::OPEN));
    h.run_steps(4);
    assert_eq!(asked.borrow().len(), 2);
    let st = h.state();
    assert_eq!(st.app.book.as_ref().map(|b| b.root.clone()), Some(project.clone()), "{:?}", st.app.message);
    let recent: Vec<String> = st.shell.recent.iter().map(ToString::to_string).collect();
    assert!(
        recent.iter().any(|r| r.ends_with("Own.wsbk")) && !recent.iter().any(|r| r.contains("book.toml")),
        "the recent list names the project: {recent:?}"
    );
}
