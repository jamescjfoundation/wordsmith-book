// SPDX-License-Identifier: MIT OR Apache-2.0
//! `cargo xtask <task>` for Wordsmith Book, built on open-source-core.
//!
//! This file and `brand.rs` are the app's own. They hold only what is Book's: its data (`APP`),
//! which includes what a release carries (`package`: the core's `docs/app-releases.md` says what
//! each field is), and its two tasks of its own: `brand`, which holds the files in
//! `assets/brand/` to the brand kit's, and `fixtures`. The gate, the packaging, every shared task
//! and the list of them are in the modules under `shared`, which the core owns and
//! `cargo xtask sync` keeps identical in every app; `cargo xtask help` lists them. Book's layers
//! are the table in `AGENTS.md`, which `cargo xtask app-layers` reads.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod brand;
mod shared;

use std::path::Path;
use std::process::ExitCode;

use shared::gate::{self, App, Task};
use shared::link::Provider;
use shared::package::{FileType, Package};
use shared::util::{TaskResult, cargo};

/// What the shared tasks need to know about Book.
const APP: App = App {
    product: "Wordsmith Book",
    // The packages that build the programs Book ships, the desktop program's first. Each
    // program has its package's name.
    shipped: &["wordsmith-book", "wordsmith-book-cli"],
    package: Some(Package {
        // Also the `id` of the update check's source and the settings folder's name
        // (`apps/book/src/main.rs`, whose tests hold the two places together).
        id: "wordsmith-book",
        // Also the app id the window is given.
        app_id: "org.jcjfoundation.book",
        summary: "Make print and ebook editions of a finished book",
        categories: "Office;Publishing;",
        icons: &[
            (32, "assets/brand/icon-32.png"),
            (256, "assets/brand/icon-256.png"),
            (512, "assets/brand/icon-512.png"),
            (1024, "assets/brand/icon-1024.png"),
        ],
        // The kit's icon drawn for macOS, with the margin and corners that system expects.
        macos_icons: &[(1024, "assets/brand/icon-macos-1024.png")],
        // `apps/book/build.rs` embeds it in the Windows program.
        windows_icon: Some("assets/brand/icon.ico"),
        // The project types (decision 0002): a project is a folder, a packed project a zip.
        file_types: &[
            FileType {
                name: "Wordsmith Book project",
                extensions: &["wsbk"],
                mime: "application/x-wordsmith-book",
                folder: true,
                ..FileType::DEFAULT
            },
            FileType {
                name: "Wordsmith Book project (one file)",
                extensions: &["wsbkz"],
                mime: "application/x-wordsmith-book-packed",
                parent: "application/zip",
                ..FileType::DEFAULT
            },
        ],
        // The license texts of the fonts the designs embed (`ATTRIBUTION.md` lists them).
        license_files: &["assets/fonts"],
        // Every field is given, so there is no `..Package::DEFAULT` here; a field a later core
        // release adds is given here when Book moves to it.
    }),
    // Book builds on Manuscript's model and format crates (decision 0001). The name is what
    // `cargo xtask dev-link manuscript=<folder>` links; the address is the one `Cargo.toml`
    // names, which `cargo xtask sync` writes into `deny.toml` as Book's own source.
    providers: &[Provider { name: "manuscript", git: "https://github.com/jamescjfoundation/wordsmith-manuscript" }],
    // `cargo xtask screenshots` redraws docs/images from a public-domain sample
    // (`crates/book-ui/examples/screenshots.rs`).
    screenshots: Some("wordsmith-book-ui"),
    tasks: TASKS,
    ..App::DEFAULT
};

/// Book's own tasks. `brand` is in the gate and reads only files; `brand --kit` asks the
/// wordsmith repository whether the kit has moved, and a workflow runs it every week.
const TASKS: &[Task] = &[
    Task::check(
        "brand",
        "the files in assets/brand/ are the brand kit's, unchanged, and the theme holds its palette (--kit: the kit has not moved)",
        brand::task,
    ),
    Task::other("fixtures", "write the fixtures that need Rust: the Manuscript project", fixtures),
];

fn main() -> ExitCode {
    gate::main(&APP)
}

/// The Manuscript project fixture: its source text and the folder written from it.
const MANUSCRIPT_FIXTURE: (&str, &str) = ("tests/fixtures/novels/gullivers-travels.txt", "tests/fixtures/novels/gullivers-travels.wsm");

/// `cargo xtask fixtures`: the Manuscript project fixture, written through Manuscript's public
/// writer. The other fixtures come from `tools/make-fixtures.py`.
fn fixtures(root: &Path, _: &[String]) -> TaskResult {
    let (src, out) = MANUSCRIPT_FIXTURE;
    cargo(root, &["run", "--quiet", "-p", "wordsmith-book-format", "--example", "manuscript_fixture", "--", src, out])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_task_table_is_sound() {
        assert_eq!(gate::table_problems(&APP), Vec::<String>::new());
        let in_gate = gate::gate_names(&APP);
        for check in ["app-layers", "assets", "release-data", "brand"] {
            assert!(in_gate.contains(&check), "{check} is not in ci");
        }
        assert!(!in_gate.contains(&"fixtures"));
    }

    /// The release names every reader of a release relies on (the core's `docs/app-releases.md`).
    /// `apps/book/src/main.rs` has the other half: the program gives the same two ids.
    #[test]
    fn the_release_carries_books_names() {
        assert_eq!(APP.shipped, ["wordsmith-book", "wordsmith-book-cli"]);
        let package = APP.package.as_ref().map(|p| (p.id, p.app_id));
        assert_eq!(package, Some(("wordsmith-book", "org.jcjfoundation.book")));
        assert_eq!(shared::package::data_problems(&APP), Vec::<String>::new());
    }

    #[test]
    fn the_provider_is_the_address_the_manifest_names() {
        let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml")).unwrap();
        for provider in APP.providers {
            assert!(manifest.contains(&format!("git = \"{}\"", provider.git)), "Cargo.toml names no dependency from {}", provider.git);
        }
    }
}
