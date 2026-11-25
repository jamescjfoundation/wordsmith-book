// SPDX-License-Identifier: MIT OR Apache-2.0
//! PDF standards (PRT-12): a print edition set to declare PDF/A-2b and PDF/UA-1 carries the
//! declarations ([`declares`]), and a PDF validator the user installed (its program in the
//! preferences) checks the file against both, as a separate program with the time limit the
//! other checkers have. Book runs `program -f 2b --format text -v <file.pdf>` and the same with
//! `ua1`, and reads each report's first word (`PASS` or `FAIL`) and its failed rules
//! (`FAIL <clause>-<test>`). Not set up is a note, never a failure.

use std::path::Path;
use std::process::Command;

use wordsmith_book_model::Severity;

use crate::preflight::{Finding, Link};
use crate::validator::{self, MAX_FINDINGS};

/// The standards checked, with the validator's names for them.
pub const STANDARDS: [(&str, &str); 2] = [("2b", "PDF/A-2b"), ("ua1", "PDF/UA-1")];

fn finding(severity: Severity, message: String) -> Finding {
    Finding { rule: "pdf-standards", severity, message, link: Link::None }
}

/// What the PDF's metadata is missing of the two declarations (none: both there).
pub fn declares(pdf: &[u8]) -> Vec<&'static str> {
    let has = |s: &str| pdf.windows(s.len()).any(|w| w == s.as_bytes());
    let mut out = Vec::new();
    if !(has("<pdfaid:part>2</pdfaid:part>") && has("/OutputIntents")) {
        out.push("PDF/A-2b");
    }
    if !(has("<pdfuaid:part>1</pdfuaid:part>") && has("/DisplayDocTitle true") && has("/StructTreeRoot")) {
        out.push("PDF/UA-1");
    }
    out
}

/// A validator's text report as findings: none when it passed.
pub fn parse(standard: &str, report: &str) -> Result<Vec<Finding>, String> {
    let first = report.split_whitespace().next().unwrap_or("");
    match first {
        "PASS" => Ok(Vec::new()),
        "FAIL" => {
            let rules: Vec<&str> = report
                .lines()
                .skip(1)
                .filter_map(|l| l.trim().strip_prefix("FAIL "))
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .take(MAX_FINDINGS)
                .collect();
            Ok(if rules.is_empty() {
                vec![finding(Severity::Blocks, format!("The PDF validator says the PDF is not {standard}."))]
            } else {
                rules
                    .iter()
                    .map(|r| finding(Severity::Blocks, format!("The PDF validator says the PDF is not {standard}: rule {r} fails.")))
                    .collect()
            })
        }
        _ => Err("its report is not one Book can read".into()),
    }
}

fn command(s: &validator::Settings, flavour: &str, pdf: &Path) -> Option<Command> {
    let program = Path::new(s.pdf.trim());
    if s.pdf.trim().is_empty() || !program.is_file() {
        return None;
    }
    let mut c = Command::new(program);
    c.args(["-f", flavour, "--format", "text", "-v"]).arg(pdf);
    // The validator runs on Java: the one the preferences name, when they name one.
    let java = Path::new(s.java.trim());
    if let Some(bin) = java.parent().filter(|_| !s.java.trim().is_empty())
        && let Some(home) = bin.parent()
    {
        let path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![bin.to_path_buf()];
        paths.extend(std::env::split_paths(&path));
        if let Ok(joined) = std::env::join_paths(paths) {
            c.env("PATH", joined);
        }
        c.env("JAVA_HOME", home);
    }
    Some(c)
}

/// Check a PDF against both standards with the validator in the preferences.
pub fn check(s: &validator::Settings, pdf: &[u8]) -> Vec<Finding> {
    if command(s, "2b", Path::new("x.pdf")).is_none() {
        return vec![finding(
            Severity::Note,
            "No PDF validator is set up, so the PDF/A-2b and PDF/UA-1 declarations were checked but not the files. Set its path in Preferences to run it too."
                .into(),
        )];
    }
    let dir = std::env::temp_dir().join(format!("wordsmith-book-pdf-{}-{}", std::process::id(), validator::next_id()));
    if std::fs::create_dir_all(&dir).is_err() {
        return vec![finding(Severity::Warning, "The PDF validator did not run: no temporary folder.".into())];
    }
    let path = dir.join("book.pdf");
    let mut out = Vec::new();
    if let Err(e) = std::fs::write(&path, pdf) {
        out.push(finding(Severity::Warning, format!("The PDF validator did not run: {e}.")));
    } else {
        for (flavour, standard) in STANDARDS {
            let Some(cmd) = command(s, flavour, &path) else { continue };
            match validator::run_limited(cmd, s.limit()) {
                Ok((report, _)) => match parse(standard, &report) {
                    Ok(f) => out.extend(f),
                    Err(m) => out.push(finding(Severity::Warning, format!("The PDF validator did not run for {standard}: {m}."))),
                },
                Err(validator::Stopped::TimedOut(d)) => out.push(finding(
                    Severity::Warning,
                    format!("The PDF validator ran for more than {} s on {standard} and was stopped; its checks did not finish.", d.as_secs()),
                )),
                Err(validator::Stopped::Failed(m)) => out.push(finding(Severity::Warning, format!("The PDF validator could not be started: {m}."))),
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_are_read() {
        assert!(parse("PDF/A-2b", "PASS /tmp/a.pdf 2b\n").unwrap().is_empty());
        let f = parse("PDF/UA-1", "FAIL /tmp/a.pdf ua1\n  FAIL 7.1-10\n  FAIL 5-1\n").unwrap();
        assert_eq!(f.len(), 2);
        assert!(f[0].message.contains("7.1-10") && f[0].severity == Severity::Blocks);
        assert_eq!(parse("PDF/UA-1", "FAIL /tmp/a.pdf ua1\n").unwrap().len(), 1);
        assert!(parse("PDF/A-2b", "Exception in thread main").is_err());
        assert_eq!(declares(b"%PDF-1.7 nothing"), ["PDF/A-2b", "PDF/UA-1"]);
    }
}
