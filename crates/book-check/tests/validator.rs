// SPDX-License-Identifier: MIT OR Apache-2.0
//! The external EPUB validator (plan M8.2, PRE-2): its report read into findings, a validator that
//! is not set up is a note, and one that runs past its time limit is stopped.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use wordsmith_book_check::preflight::Link;
use wordsmith_book_check::validator::{self, Outcome, Settings};
use wordsmith_book_model::Severity;

/// A report in the validator's usual form, written for this test.
const SAMPLE: &str = "Validating using EPUB version 3.3 rules.
ERROR(RSC-005): book.epub/EPUB/text/s006.xhtml(12,40): Error while parsing file: element \"x\" not allowed here
WARNING(RSC-017): book.epub/EPUB/text/s029.xhtml(35,57): A role is deprecated and should not be used.
ERROR(PKG-007): book.epub/mimetype: Mimetype file should only contain the string \"application/epub+zip\".
FATAL(RSC-016): book.epub: Fatal Error while parsing file: unexpected end of file
INFO(ACC-009): book.epub/EPUB/package.opf: something worth knowing

Check finished with errors
Messages: 1 fatal / 2 errors / 1 warning / 1 info
";

fn tmp(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_report_becomes_findings_with_their_files() {
    let f = validator::parse(SAMPLE);
    assert_eq!(f.len(), 5, "{f:#?}");
    assert_eq!(f[0].severity, Severity::Blocks);
    assert_eq!(f[0].link, Link::File("EPUB/text/s006.xhtml".into()));
    assert!(f[0].message.contains("not allowed here") && f[0].message.contains("12,40") && f[0].message.contains("RSC-005"), "{}", f[0].message);
    assert_eq!(f[1].severity, Severity::Warning);
    assert_eq!(f[2].link, Link::File("mimetype".into()));
    assert_eq!(f[3].severity, Severity::Blocks);
    assert_eq!(f[3].link, Link::None);
    assert_eq!(f[4].severity, Severity::Note);
    assert!(validator::parse("Messages: 0 fatals / 0 errors\nCheck completed\n").is_empty());
}

#[test]
fn a_validator_that_is_not_set_up_is_a_note() {
    for s in [Settings::default(), Settings { program: "/nowhere/at/all.jar".into(), ..Settings::default() }] {
        assert_eq!(validator::run(&s, Path::new("book.epub")), Outcome::NotSetUp);
        let f = validator::check(&s, b"PK");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Note);
        assert!(f[0].message.contains("not set up"));
    }
}

#[cfg(unix)]
fn script(dir: &Path, name: &str, body: &str) -> String {
    use std::os::unix::fs::PermissionsExt as _;
    let p = dir.join(name);
    std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p.display().to_string()
}

#[cfg(unix)]
#[test]
fn a_program_that_reports_is_read_and_one_that_runs_too_long_is_stopped() {
    let dir = tmp("validator");
    std::fs::write(dir.join("report.txt"), SAMPLE).unwrap();
    let reporting = script(&dir, "reporting", &format!("cat '{}'", dir.join("report.txt").display()));
    let f = validator::check(&Settings { program: reporting, ..Settings::default() }, b"PK");
    assert_eq!(f.len(), 5);
    assert_eq!(f.iter().filter(|x| x.severity == Severity::Blocks).count(), 3);
    let slow = script(&dir, "slow", "sleep 30");
    let start = Instant::now();
    let o = validator::run(&Settings { program: slow, seconds: 1, ..Settings::default() }, &dir.join("book.epub"));
    assert_eq!(o, Outcome::TimedOut(Duration::from_secs(1)));
    assert!(start.elapsed() < Duration::from_secs(10), "stopped promptly");
    let f = validator::findings(&o);
    assert_eq!(f[0].severity, Severity::Warning);
    assert!(f[0].message.contains("was stopped"));
    // A program that is not a program.
    let not = dir.join("not-a-program");
    std::fs::write(&not, "plain text").unwrap();
    assert!(matches!(
        validator::run(&Settings { program: not.display().to_string(), ..Settings::default() }, &dir.join("book.epub")),
        Outcome::Failed(_)
    ));
}

#[test]
fn the_real_validator_finds_nothing_in_a_book_epub_where_it_is_set_up() {
    let Some(jar) = std::env::var_os("BOOK_EPUB_VALIDATOR") else {
        eprintln!("the standard EPUB validator is not set up (BOOK_EPUB_VALIDATOR); skipped");
        return;
    };
    let java = std::env::var("BOOK_JAVA").unwrap_or_default();
    let novels = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels");
    let (_, p) = wordsmith_manuscript_format::folder::Folder::open(&novels.join("gullivers-travels.wsm")).unwrap();
    let book = wordsmith_manuscript_format::book::from_project(&p);
    let mut project = wordsmith_book_model::BookProject::new(wordsmith_book_model::Source::default(), "2026-11-24");
    project.elements = wordsmith_book_model::matter::default_elements();
    let (_, e) =
        wordsmith_book_check::build::ebook(&project, &Default::default(), &book, &wordsmith_book_model::Edition::ebook(), &|_| None).unwrap();
    let f = validator::check(&Settings { program: jar.to_string_lossy().into_owned(), java, seconds: 120, access: String::new() }, &e.bytes);
    assert!(f.iter().all(|x| x.severity == Severity::Note), "{f:#?}");
}
