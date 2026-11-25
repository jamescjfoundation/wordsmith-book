// SPDX-License-Identifier: MIT OR Apache-2.0
//! External validators (PRE-2, SPEC 8.7): the standard EPUB validator, when the user installed
//! it and set its path in the preferences, runs as a separate program with a time limit, and
//! its report becomes preflight findings. "Not set up" is a note, never a failure.
//!
//! The program is either the validator's Java archive (`.jar`, run with the Java runtime the
//! preferences name, else `java`) or any program that takes the EPUB's path as its argument and
//! writes the validator's usual report.

use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wordsmith_book_model::Severity;

use crate::preflight::{Finding, Link};

/// Most of a validator's report read, in bytes.
pub const MAX_REPORT: u64 = 8 << 20;
/// Most findings kept from one report.
pub const MAX_FINDINGS: usize = 200;

/// Where the validator is and how long it may run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// The validator: its `.jar`, or a program. Empty when not set up.
    pub program: String,
    /// The Java runtime for a `.jar` (empty: `java` on the path).
    pub java: String,
    /// The time limit, in seconds (0: two minutes), for each program.
    pub seconds: u32,
    /// The accessibility checker (see [`crate::access_check`]). Empty when not set up.
    pub access: String,
    /// The PDF validator (see [`crate::pdf_check`]). Empty when not set up.
    pub pdf: String,
}

impl Settings {
    /// The time limit.
    pub fn limit(&self) -> Duration {
        Duration::from_secs(if self.seconds == 0 { 120 } else { u64::from(self.seconds.min(3600)) })
    }
}

/// Why a program gave no report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stopped {
    /// It ran past the time limit and was stopped.
    TimedOut(Duration),
    /// It could not be started or waited for.
    Failed(String),
}

/// A number for a temporary folder, different on every call.
pub fn next_id() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Run a program with a time limit, its output (standard output, then errors) read on threads
/// so a long report cannot fill a pipe and stall it, each bounded.
pub fn run_limited(mut cmd: Command, limit: Duration) -> Result<(String, String), Stopped> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| Stopped::Failed(e.to_string()))?;
    let read = |pipe: Option<Box<dyn std::io::Read + Send>>| {
        std::thread::spawn(move || {
            let mut out = Vec::new();
            if let Some(p) = pipe {
                let _ = p.take(MAX_REPORT).read_to_end(&mut out);
            }
            String::from_utf8_lossy(&out).into_owned()
        })
    };
    let out = read(child.stdout.take().map(|p| Box::new(p) as Box<dyn std::io::Read + Send>));
    let err = read(child.stderr.take().map(|p| Box::new(p) as Box<dyn std::io::Read + Send>));
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() >= limit => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Stopped::TimedOut(limit));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(Stopped::Failed(e.to_string())),
        }
    }
    Ok((out.join().unwrap_or_default(), err.join().unwrap_or_default()))
}

/// What running the validator gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// No validator is set up, or its file is not there.
    NotSetUp,
    /// It ran: what it found, and whether it said the EPUB passed.
    Ran(Vec<Finding>),
    /// It ran past the time limit and was stopped.
    TimedOut(Duration),
    /// It could not be started or its report could not be read.
    Failed(String),
}

/// One line of the validator's report, as a finding: `ERROR(RSC-005): book.epub/EPUB/text/a.xhtml(12,5): message`.
pub fn parse_line(line: &str) -> Option<Finding> {
    let line = line.trim();
    let (level, rest) = line.split_once('(')?;
    let severity = match level {
        "FATAL" | "ERROR" => Severity::Blocks,
        "WARNING" => Severity::Warning,
        "INFO" | "USAGE" => Severity::Note,
        _ => return None,
    };
    let (code, rest) = rest.split_once("):")?;
    let rest = rest.trim();
    // The place: `file.epub/path(line,col)`, or `file.epub/path`, then `: message`.
    let (place, message) = match rest.find("): ") {
        Some(i) if rest[..i].contains('(') => (&rest[..=i], rest[i + 3..].trim()),
        _ => match rest.split_once(": ") {
            Some((p, m)) if p.contains(".epub") => (p, m.trim()),
            _ => ("", rest),
        },
    };
    let file = place.split_once(".epub/").map(|(_, f)| f.split('(').next().unwrap_or(f).to_owned());
    let at = place.rsplit_once('(').map(|(_, p)| p.trim_end_matches(')').to_owned());
    let mut text = format!("The EPUB validator: {message}");
    if let Some(at) = at.filter(|a| !a.is_empty() && a.chars().all(|c| c.is_ascii_digit() || c == ',' || c == '-')) {
        text.push_str(&format!(" (line and column {at})"));
    }
    text.push_str(&format!(" [{code}]"));
    Some(Finding { rule: "validator", severity, message: text, link: file.filter(|f| !f.is_empty()).map_or(Link::None, Link::File) })
}

/// Every finding in a validator's report, bounded.
pub fn parse(report: &str) -> Vec<Finding> {
    report.lines().filter_map(parse_line).take(MAX_FINDINGS).collect()
}

fn command(s: &Settings, epub: &Path) -> Option<Command> {
    if s.program.trim().is_empty() {
        return None;
    }
    let program = PathBuf::from(s.program.trim());
    if !program.is_file() {
        return None;
    }
    let mut c = if program.extension().is_some_and(|e| e.eq_ignore_ascii_case("jar")) {
        let java = if s.java.trim().is_empty() { PathBuf::from("java") } else { PathBuf::from(s.java.trim()) };
        let mut c = Command::new(java);
        c.arg("-jar").arg(&program);
        c
    } else {
        Command::new(&program)
    };
    c.arg(epub);
    Some(c)
}

/// Run the validator on an EPUB file.
pub fn run(s: &Settings, epub: &Path) -> Outcome {
    let Some(cmd) = command(s, epub) else { return Outcome::NotSetUp };
    match run_limited(cmd, s.limit()) {
        Ok((out, err)) => Outcome::Ran(parse(&(out + &err))),
        Err(Stopped::TimedOut(d)) => Outcome::TimedOut(d),
        Err(Stopped::Failed(m)) => Outcome::Failed(format!("the validator could not be started: {m}")),
    }
}

/// The findings an outcome adds to preflight.
pub fn findings(o: &Outcome) -> Vec<Finding> {
    let note = |severity, m: String| vec![Finding { rule: "validator", severity, message: m, link: Link::None }];
    match o {
        Outcome::NotSetUp => note(
            Severity::Note,
            "The standard EPUB validator is not set up, so only Book's own checks ran. Set its path in Preferences to run it too.".into(),
        ),
        Outcome::Ran(f) => f.clone(),
        Outcome::TimedOut(d) => {
            note(Severity::Warning, format!("The EPUB validator ran for more than {} s and was stopped; its checks did not finish.", d.as_secs()))
        }
        Outcome::Failed(m) => note(Severity::Warning, format!("The EPUB validator did not run: {m}.")),
    }
}

/// Write an EPUB to a temporary file, run the validator on it, and give the findings.
pub fn check(s: &Settings, epub: &[u8]) -> Vec<Finding> {
    if command(s, Path::new("x.epub")).is_none() {
        return findings(&Outcome::NotSetUp);
    }
    let dir = std::env::temp_dir().join(format!("wordsmith-book-validate-{}-{}", std::process::id(), next_id()));
    if std::fs::create_dir_all(&dir).is_err() {
        return findings(&Outcome::Failed("no temporary folder".into()));
    }
    let path = dir.join("book.epub");
    if let Err(e) = std::fs::write(&path, epub) {
        return findings(&Outcome::Failed(e.to_string()));
    }
    let out = findings(&run(s, &path));
    let _ = std::fs::remove_file(&path);
    out
}
