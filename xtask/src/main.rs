// SPDX-License-Identifier: MIT OR Apache-2.0
//! `cargo xtask <task>` for Wordsmith Book, built on open-source-core.
//!
//! This file is the app's own, and it holds only what is Book's: its data (`APP`) and its own
//! task (`fixtures`). The gate, every shared task and the list of them are in the modules under
//! `shared`, which the core owns and `cargo xtask sync` keeps identical in every app;
//! `cargo xtask help` lists them. Book's layers are the table in `AGENTS.md`, which
//! `cargo xtask app-layers` reads.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod shared;

use std::path::Path;
use std::process::ExitCode;

use shared::gate::{self, App, Task};
use shared::link::Provider;
use shared::util::{TaskResult, cargo};

/// What the shared tasks need to know about Book.
const APP: App = App {
    product: "Wordsmith Book",
    // The programs Book ships (for THIRD-PARTY-LICENSES.txt).
    shipped: &["wordsmith-book", "book-cli"],
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

/// Book's own tasks.
const TASKS: &[Task] = &[Task::other("fixtures", "write the fixtures that need Rust: the Manuscript project", fixtures)];

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
        assert!(in_gate.contains(&"app-layers") && !in_gate.contains(&"fixtures"));
    }

    #[test]
    fn the_provider_is_the_address_the_manifest_names() {
        let manifest = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml")).unwrap();
        for provider in APP.providers {
            assert!(manifest.contains(&format!("git = \"{}\"", provider.git)), "Cargo.toml names no dependency from {}", provider.git);
        }
    }
}
