// SPDX-License-Identifier: MIT OR Apache-2.0
//! The accessibility checker (plan R2.14, PRE-2): its report read into findings with their
//! files, a checker that is not set up is a note, one that runs past its time limit is stopped,
//! and one that gives no report it can read is a warning.

use std::path::Path;

use wordsmith_book_check::access_check;
use wordsmith_book_check::preflight::Link;
use wordsmith_book_check::validator::Settings;
use wordsmith_book_model::Severity;

/// A report in the checker's usual form, written for this test: one problem in the package, two
/// in a chapter (one of them repeated), a test that passed, and one problem without a file.
const SAMPLE: &str = r#"{
  "@type": "earl:report",
  "dct:title": "Accessibility report",
  "earl:testSubject": { "url": "book.epub", "dct:title": "A Book" },
  "assertions": [
    {
      "@type": "earl:assertion",
      "earl:testSubject": { "url": "EPUB/package.opf", "dct:title": "A Book" },
      "assertions": [
        {
          "@type": "earl:assertion",
          "earl:result": { "earl:outcome": "fail", "dct:description": "Publications with page breaks must say where they come from" },
          "earl:test": { "earl:impact": "serious", "dct:title": "epub-pagesource", "dct:description": "Ensures the source of page breaks is identified" }
        }
      ]
    },
    {
      "@type": "earl:assertion",
      "earl:testSubject": { "url": "EPUB/text/s003.xhtml#start", "dct:title": "Chapter One" },
      "assertions": [
        {
          "@type": "earl:assertion",
          "earl:result": { "earl:outcome": "fail", "dct:description": "Fix any of the following:\n  Element has insufficient color contrast" },
          "earl:test": { "earl:impact": "critical", "dct:title": "color-contrast" }
        },
        {
          "@type": "earl:assertion",
          "earl:result": { "earl:outcome": "fail", "dct:description": "Fix any of the following:\n  Element has insufficient color contrast" },
          "earl:test": { "earl:impact": "critical", "dct:title": "color-contrast" }
        },
        {
          "@type": "earl:assertion",
          "earl:result": { "earl:outcome": "pass" },
          "earl:test": { "earl:impact": "minor", "dct:title": "region" }
        },
        {
          "@type": "earl:assertion",
          "earl:result": { "earl:outcome": "fail", "dct:description": "Headings should not skip levels" },
          "earl:test": { "earl:impact": "moderate", "dct:title": "heading-order" }
        }
      ]
    }
  ],
  "earl:result": { "earl:outcome": "fail" }
}"#;

#[test]
fn the_report_becomes_findings_with_their_files() {
    let f = access_check::parse(SAMPLE).unwrap();
    assert_eq!(f.len(), 3, "a passed test and a repeat are left out: {f:#?}");
    assert_eq!(f[0].severity, Severity::Warning);
    assert_eq!(f[0].link, Link::File("EPUB/package.opf".into()));
    assert!(f[0].message.contains("say where they come from") && f[0].message.contains("[epub-pagesource]"), "{}", f[0].message);
    assert_eq!(f[1].severity, Severity::Blocks);
    assert_eq!(f[1].link, Link::File("EPUB/text/s003.xhtml".into()));
    assert!(f[1].message.contains("Fix any of the following: Element has"), "lines joined: {}", f[1].message);
    assert_eq!(f[2].severity, Severity::Note);
    assert!(f.iter().all(|x| x.rule == "access-check"));
    // A report with nothing wrong, and reports Book cannot read.
    assert!(access_check::parse(r#"{"assertions": [], "earl:result": {"earl:outcome": "pass"}}"#).unwrap().is_empty());
    assert!(access_check::parse("Checking ...\nDone.").is_err());
    assert!(access_check::parse("[1, 2]").is_err());
}

#[test]
fn a_checker_that_is_not_set_up_is_a_note() {
    for s in [Settings::default(), Settings { access: "/nowhere/at/all".into(), ..Settings::default() }] {
        let f = access_check::check(&s, b"PK");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Note);
        assert!(f[0].message.contains("Preferences"), "{}", f[0].message);
    }
}

/// The tests that run a program in the checker's place. The stand-ins are shell scripts, so the
/// whole module is for Unix: on Windows nothing in it is built, and nothing outside it is left
/// unused there. What the tests show holds on every system: the program is started the same way.
#[cfg(unix)]
mod with_a_stand_in_program {
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    use wordsmith_book_check::access_check::Outcome;

    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn script(dir: &Path, name: &str, body: &str) -> String {
        use std::os::unix::fs::PermissionsExt as _;
        let p = dir.join(name);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p.display().to_string()
    }

    #[test]
    fn a_checker_that_reports_is_read_and_one_that_runs_too_long_is_stopped() {
        let dir = tmp("access-check");
        std::fs::write(dir.join("sample.json"), SAMPLE).unwrap();
        // The checker's usual way: `-f -s -o <folder> <epub>`, the report in the folder.
        let writes = script(
            &dir,
            "writes",
            &format!("[ \"$3\" = -o ] || exit 9\nmkdir -p \"$4\" && cp '{}' \"$4/report.json\"", dir.join("sample.json").display()),
        );
        let f = access_check::check(&Settings { access: writes, ..Settings::default() }, b"PK");
        assert_eq!(f.len(), 3, "{f:#?}");
        assert_eq!(f.iter().filter(|x| x.severity == Severity::Blocks).count(), 1);
        // A checker that writes its report to its output instead.
        let prints = script(&dir, "prints", &format!("cat '{}'", dir.join("sample.json").display()));
        assert_eq!(access_check::check(&Settings { access: prints, ..Settings::default() }, b"PK").len(), 3);
        // One that gives nothing Book can read.
        let silent = script(&dir, "silent", "echo 'something went wrong' >&2; exit 1");
        let f = access_check::check(&Settings { access: silent, ..Settings::default() }, b"PK");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Warning);
        assert!(f[0].message.contains("did not run"), "{}", f[0].message);
        // One that runs too long.
        let slow = script(&dir, "slow", "sleep 30");
        let start = Instant::now();
        let o = access_check::run(&Settings { access: slow, seconds: 1, ..Settings::default() }, &dir.join("book.epub"), &dir.join("out"));
        assert_eq!(o, Outcome::TimedOut(Duration::from_secs(1)));
        assert!(start.elapsed() < Duration::from_secs(10), "stopped promptly");
        let f = access_check::findings(&o);
        assert_eq!(f[0].severity, Severity::Warning);
        assert!(f[0].message.contains("was stopped"));
    }
}

#[test]
fn the_real_checker_finds_nothing_that_matters_in_a_book_epub_where_it_is_set_up() {
    let Some(program) = std::env::var_os("BOOK_ACCESS_CHECKER") else {
        eprintln!("no accessibility checker is set up (BOOK_ACCESS_CHECKER); skipped");
        return;
    };
    let novels = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/novels");
    let (_, p) = wordsmith_manuscript_format::folder::Folder::open(&novels.join("gullivers-travels.wsm")).unwrap();
    let book = wordsmith_manuscript_format::book::from_project(&p);
    let mut project = wordsmith_book_model::BookProject::new(wordsmith_book_model::Source::default(), "2026-11-24");
    project.elements = wordsmith_book_model::matter::default_elements();
    let (_, e) =
        wordsmith_book_check::build::ebook(&project, &Default::default(), &book, &wordsmith_book_model::Edition::ebook(), &|_| None).unwrap();
    let s = Settings { access: program.to_string_lossy().into_owned(), seconds: 600, ..Settings::default() };
    let f = access_check::check(&s, &e.bytes);
    assert!(f.iter().all(|x| x.severity == Severity::Note), "{f:#?}");
}
