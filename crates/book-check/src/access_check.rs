// SPDX-License-Identifier: MIT OR Apache-2.0
//! The accessibility checker (PRE-2, SPEC 8.7): an EPUB accessibility checker the user installed
//! and named in the preferences runs as a separate program with a time limit, and the problems
//! in its report become preflight findings. "Not set up" is a note, never a failure.
//!
//! Book runs `program -f -s -o <folder> <book.epub>` and reads `<folder>/report.json`, the
//! checker's usual report: assertions in the W3C evaluation-report vocabulary (`earl:`), each
//! problem an assertion whose result is `fail`, with the test's title and impact (`critical`,
//! `serious`, `moderate`, `minor`) and the file it was found in. When the folder holds no report,
//! the program's own output is read as the report.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde_json::Value;
use wordsmith_book_model::Severity;

use crate::preflight::{Finding, Link};
use crate::validator::{self, MAX_FINDINGS, MAX_REPORT};

/// How deep the report's nesting is followed.
const MAX_DEPTH: usize = 32;

/// What running the checker gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// No checker is set up, or its file is not there.
    NotSetUp,
    /// It ran: the problems it reported.
    Ran(Vec<Finding>),
    /// It ran past the time limit and was stopped.
    TimedOut(Duration),
    /// It could not be started, or its report could not be read.
    Failed(String),
}

fn text(v: &Value, key: &str) -> Option<String> {
    match v.get(key)? {
        Value::String(s) => Some(s.trim().to_owned()),
        Value::Array(a) => a.iter().find_map(|x| x.as_str()).map(|s| s.trim().to_owned()),
        _ => None,
    }
    .filter(|s| !s.is_empty())
}

fn severity(impact: &str) -> Severity {
    match impact.to_ascii_lowercase().as_str() {
        "critical" => Severity::Blocks,
        "serious" => Severity::Warning,
        _ => Severity::Note,
    }
}

/// The file a test subject names, as a path inside the EPUB.
fn file_of(subject: &Value) -> Option<String> {
    let url = text(subject, "url").or_else(|| text(subject, "dct:title"))?;
    let url = url.split('#').next().unwrap_or(&url);
    let path = url.split_once(".epub/").map_or(url, |(_, p)| p);
    (!path.is_empty() && !path.contains("://") && !path.ends_with(".epub")).then(|| path.trim_start_matches('/').to_owned())
}

fn one(assertion: &Value, file: Option<&str>) -> Option<Finding> {
    let result = assertion.get("earl:result")?;
    if !text(result, "earl:outcome")?.eq_ignore_ascii_case("fail") {
        return None;
    }
    let test = assertion.get("earl:test")?;
    let rule = text(test, "dct:title").unwrap_or_else(|| "unnamed".into());
    let impact = text(test, "earl:impact").unwrap_or_default();
    let what = text(result, "dct:description").or_else(|| text(test, "dct:description")).unwrap_or_else(|| rule.clone());
    let what = what.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ");
    let what: String = what.chars().take(400).collect();
    let mut message = format!("The accessibility checker: {what}");
    if !impact.is_empty() {
        message.push_str(&format!(" ({impact})"));
    }
    message.push_str(&format!(" [{rule}]"));
    Some(Finding { rule: "access-check", severity: severity(&impact), message, link: file.map_or(Link::None, |f| Link::File(f.to_owned())) })
}

fn walk(v: &Value, file: Option<&str>, depth: usize, out: &mut Vec<Finding>) {
    if depth > MAX_DEPTH || out.len() >= MAX_FINDINGS {
        return;
    }
    match v {
        Value::Object(map) => {
            let here = map.get("earl:testSubject").and_then(file_of);
            let file = here.as_deref().or(file);
            if map.contains_key("earl:test") {
                out.extend(one(v, file));
            }
            for (k, child) in map {
                if k != "earl:result" && k != "earl:test" {
                    walk(child, file, depth + 1, out);
                }
            }
        }
        Value::Array(a) => {
            for child in a {
                walk(child, file, depth + 1, out);
            }
        }
        _ => {}
    }
}

/// The problems in a checker's report, bounded, in the report's order with repeats dropped.
pub fn parse(report: &str) -> Result<Vec<Finding>, String> {
    let v: Value = serde_json::from_str(report.trim()).map_err(|e| format!("its report is not one Book can read ({e})"))?;
    if !v.is_object() {
        return Err("its report is not one Book can read".into());
    }
    let mut out = Vec::new();
    walk(&v, None, 0, &mut out);
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|f| seen.insert((f.message.clone(), format!("{:?}", f.link))));
    Ok(out)
}

fn command(program: &str, epub: &Path, folder: &Path) -> Option<Command> {
    let program = PathBuf::from(program.trim());
    if program.as_os_str().is_empty() || !program.is_file() {
        return None;
    }
    let mut c = Command::new(&program);
    c.arg("-f").arg("-s").arg("-o").arg(folder).arg(epub);
    Some(c)
}

/// Run the checker on an EPUB file, its report written to `folder`.
pub fn run(s: &validator::Settings, epub: &Path, folder: &Path) -> Outcome {
    let Some(cmd) = command(&s.access, epub, folder) else { return Outcome::NotSetUp };
    let (stdout, _) = match validator::run_limited(cmd, s.limit()) {
        Ok(o) => o,
        Err(validator::Stopped::TimedOut(d)) => return Outcome::TimedOut(d),
        Err(validator::Stopped::Failed(m)) => return Outcome::Failed(format!("the accessibility checker could not be started: {m}")),
    };
    let report = std::fs::File::open(folder.join("report.json")).ok().and_then(|f| {
        use std::io::Read as _;
        let mut s = String::new();
        f.take(MAX_REPORT).read_to_string(&mut s).ok().map(|_| s)
    });
    match parse(report.as_deref().unwrap_or(&stdout)) {
        Ok(f) => Outcome::Ran(f),
        Err(m) => Outcome::Failed(m),
    }
}

/// The findings an outcome adds to preflight.
pub fn findings(o: &Outcome) -> Vec<Finding> {
    let note = |severity, m: String| vec![Finding { rule: "access-check", severity, message: m, link: Link::None }];
    match o {
        Outcome::NotSetUp => note(
            Severity::Note,
            "No accessibility checker is set up, so only Book's own accessibility checks ran. Set its path in Preferences to run it too.".into(),
        ),
        Outcome::Ran(f) => f.clone(),
        Outcome::TimedOut(d) => note(
            Severity::Warning,
            format!("The accessibility checker ran for more than {} s and was stopped; its checks did not finish.", d.as_secs()),
        ),
        Outcome::Failed(m) => note(Severity::Warning, format!("The accessibility checker did not run: {m}.")),
    }
}

/// Write an EPUB to a temporary folder, run the checker on it, and give the findings.
pub fn check(s: &validator::Settings, epub: &[u8]) -> Vec<Finding> {
    if command(&s.access, Path::new("x.epub"), Path::new("x")).is_none() {
        return findings(&Outcome::NotSetUp);
    }
    let dir = std::env::temp_dir().join(format!("wordsmith-book-access-{}-{}", std::process::id(), validator::next_id()));
    if std::fs::create_dir_all(&dir).is_err() {
        return findings(&Outcome::Failed("no temporary folder".into()));
    }
    let path = dir.join("book.epub");
    let out = match std::fs::write(&path, epub) {
        Ok(()) => findings(&run(s, &path, &dir.join("report"))),
        Err(e) => findings(&Outcome::Failed(e.to_string())),
    };
    let _ = std::fs::remove_dir_all(&dir);
    out
}
