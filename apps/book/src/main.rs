// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book, the desktop app: the core's shell around Book's views (`wordsmith-book-ui`).
//!
//! `wordsmith-book [project]` opens the project (a `.wsbk` folder, its `book.toml` or a packed
//! `.wsbkz`). `wordsmith-book --version` prints the program's name and version and opens no
//! window.
//!
//! The start-up is the core's (`jcj_shell::desktop`, core decision 0014): this file gives Book's
//! data ([`DESKTOP`]), and `wordsmith_book_ui::desktop` what only Book does. The window's title,
//! app id and icon, the settings folder, the preferences, the update check, the close that
//! starts an installed update and its helper, the file dialogs and the steps of every frame and
//! of the end are not written here, so none of them can be left out. Before anything else,
//! `main` hands its arguments to the updater, because on Windows the updater finishes through a
//! copy of this program (the core's `docs/app-releases.md`, "Updating in place").

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::ffi::OsString;
use std::process::ExitCode;

use jcj_shell::desktop::{self, Desktop, Releases};
use wordsmith_book_ui::{NAME, brand};

/// The release id: the stem of every release file, the desktop program's name, and the name of
/// the settings folder on Linux. `xtask/src/main.rs` gives the packaging the same id.
const ID: &str = "wordsmith-book";

/// The id the window is given. A Linux desktop joins the window to its menu entry and icon by
/// it, and the macOS app carries it; `xtask/src/main.rs` gives the packaging the same id.
const APP_ID: &str = "org.jcjfoundation.book";

/// Book's data for the core's start-up: its names, its brand, and where its releases come from,
/// for the update check. Updates are installed in place on Linux and Windows; on macOS the
/// update check offers the download.
const DESKTOP: Desktop = Desktop {
    brand: brand::BRAND,
    releases: Some(Releases {
        repo: "jamescjfoundation/wordsmith-book",
        tagline: "Make print and ebook editions of a finished book",
        developer: "The James C. J. Foundation",
    }),
    ..Desktop::new(NAME, ID, APP_ID, env!("CARGO_PKG_VERSION"))
};

/// Whether the command line asks for the version line (`--version` or `-V` first) and no window.
fn asks_version(args: &[OsString]) -> bool {
    args.first().is_some_and(|first| first == "--version" || first == "-V")
}

/// The line `--version` prints: the program's name and its version.
fn version_line() -> String {
    format!("{ID} {}", env!("CARGO_PKG_VERSION"))
}

fn main() -> ExitCode {
    // First of all: on Windows an update is finished by a copy of this program, started with
    // the updater's own arguments. It opens no window.
    if let Some(code) = DESKTOP.finish_update() {
        return code;
    }
    if asks_version(&std::env::args_os().skip(1).collect::<Vec<_>>()) {
        println!("{}", version_line());
        return ExitCode::SUCCESS;
    }
    // The project named on the command line, kept as the system gave it, so a name that is not
    // Unicode still opens.
    match DESKTOP.run(desktop::arguments(), wordsmith_book_ui::desktop::build) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{ID}: could not open a window: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use jcj_shell::egui;

    use super::*;

    /// The names a release is found and installed by (the core's `docs/app-releases.md`). The
    /// packaging has the same two ids in `xtask/src/main.rs`, whose own test holds them there:
    /// the xtask cannot read this file, so each side says them and both are tested.
    #[test]
    fn the_program_carries_the_names_a_release_is_found_by() {
        assert_eq!(ID, "wordsmith-book");
        assert_eq!(APP_ID, "org.jcjfoundation.book");
        assert_eq!(APP_ID, wordsmith_book_ui::APP_ID);
        // The desktop program is named by the release id, and so is its package.
        assert_eq!(env!("CARGO_BIN_NAME"), ID);
        assert_eq!(env!("CARGO_PKG_NAME"), ID);
        let releases = DESKTOP.release_source().unwrap();
        assert_eq!(releases.id, ID);
        assert_eq!(releases.name, NAME);
        assert_eq!(releases.tagline, "Make print and ebook editions of a finished book");
        assert_eq!(releases.repo, "jamescjfoundation/wordsmith-book");
        assert_eq!(releases.releases_url(), "https://github.com/jamescjfoundation/wordsmith-book/releases");
        assert_eq!(releases.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(releases.developer, "The James C. J. Foundation");
    }

    #[test]
    fn macos_is_sent_to_the_release_page() {
        assert_eq!(DESKTOP.release_source().unwrap().installs_in_place, !cfg!(target_os = "macos"));
    }

    #[test]
    fn an_ordinary_start_is_not_an_updates_helper() {
        assert!(DESKTOP.finish_update_with(&[]).is_none());
        assert!(DESKTOP.finish_update_with(&[OsString::from("novel.wsbk")]).is_none());
        assert!(DESKTOP.finish_update_with(&[OsString::from("--version")]).is_none());
        // The updater's own first argument is never read as a project to open.
        assert!(DESKTOP.finish_update_with(&[OsString::from(jcj_shell::updates::desktop::FINISH_FLAG)]).is_some());
    }

    #[test]
    fn the_command_line_is_a_project_or_the_version() {
        assert!(!asks_version(&[]));
        assert!(!asks_version(&[OsString::from("novel.wsbk")]));
        assert!(asks_version(&[OsString::from("--version")]));
        assert!(asks_version(&[OsString::from("-V")]));
        assert_eq!(desktop::files_named([OsString::from("novel.wsbk")]), [PathBuf::from("novel.wsbk")]);
        assert_eq!(version_line(), concat!("wordsmith-book ", env!("CARGO_PKG_VERSION")));
    }

    /// The window is the app's name, with its app id and icon, as before the start-up was the
    /// core's.
    #[test]
    fn the_window_carries_the_app_s_name_id_and_icon() {
        let viewport = DESKTOP.viewport();
        assert_eq!(viewport.title.as_deref(), Some(NAME));
        assert_eq!(viewport.app_id.as_deref(), Some(APP_ID));
        assert!(viewport.icon.is_some(), "the brand's icon decodes");
    }

    /// The start-up without a window: Book is built with the start window on, says on its
    /// message line that a named project cannot be opened, and keeps its preferences in the
    /// settings folder, written once more at the end.
    #[test]
    fn book_starts_from_its_data() {
        let dir = std::env::temp_dir().join(format!("wordsmith-book-start-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let settings = dir.join("settings");
        let ctx = egui::Context::default();
        let missing = dir.join("Nowhere.wsbk");
        let mut window = jcj_shell::desktop::Window::start(&DESKTOP, &ctx, Some(settings.clone()), vec![missing], wordsmith_book_ui::desktop::build);
        assert!(window.app.message.is_some(), "a project that cannot be opened is said on the message line");
        assert!(window.app.book.is_none());
        assert_eq!(window.prefs_path(), Some(settings.join("preferences.json").as_path()));
        // A preference changed just before the end is written at the end.
        window.app.prefs.validator = "a-validator-for-the-test".into();
        window.exit();
        let written = std::fs::read_to_string(settings.join("preferences.json")).unwrap();
        assert!(written.contains("a-validator-for-the-test"), "{written}");
        drop(window);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
