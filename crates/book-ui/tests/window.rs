// SPDX-License-Identifier: MIT OR Apache-2.0
//! The app's window: the core's shell drawn around Book's app shows Book's name.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use jcj_shell::Shell;
use jcj_shell::egui;
use wordsmith_book_ui::{BookApp, NAME, brand, shell_options};

struct St {
    shell: Shell,
    app: BookApp,
}

#[test]
fn the_window_opens_with_the_apps_name() {
    let shell = Shell::new(NAME, "0.1.0", brand::tokens()).with_options(shell_options());
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(|ui, st: &mut St| st.shell.ui(ui, &mut st.app), St { shell, app: BookApp::new() });
    h.run();
    assert!(h.query_by_label_contains(NAME).is_some(), "the window shows the app's name");
}
