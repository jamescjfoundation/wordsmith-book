// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's own EPUB checks (plan M6.3): each check against an EPUB broken for it; every design's
//! EPUB of the v0.1 test books passes, reads back through the core's reader word for word as
//! Book's reader reads it, holds exactly the source's words, and matches its golden structure
//! (`tests/golden/epub/`, docs/golden.md). Set `BOOK_BLESS=1` to write the golden files again.
//! When `BOOK_EPUB_VALIDATOR` names the standard validator's jar (and `BOOK_JAVA` a Java runtime,
//! else `java`), it is run on each EPUB too.

mod common;

use std::io::Write as _;

use common::*;
use wordsmith_book_check::epub::{self, Problem, Rule};
use wordsmith_book_check::{build, verify};
use wordsmith_book_design::builtin;
use wordsmith_book_model::Edition;
use wordsmith_manuscript_format::book::Book;

fn ebook(b: &Book, design: &str) -> Vec<u8> {
    let mut p = project();
    p.design.id = design.into();
    build::ebook(&p, &Default::default(), b, &Edition::ebook(), &|_| None).unwrap_or_else(|e| panic!("{design}: {e}")).1.bytes
}

/// A zip of `files`, `mimetype` stored when `stored`, the rest compressed.
fn rezip(files: &[(String, bool, Vec<u8>)]) -> Vec<u8> {
    let mut z = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, stored, bytes) in files {
        let m = if *stored { zip::CompressionMethod::Stored } else { zip::CompressionMethod::Deflated };
        z.start_file(name.as_str(), zip::write::SimpleFileOptions::default().compression_method(m)).unwrap();
        z.write_all(bytes).unwrap();
    }
    z.finish().unwrap().into_inner()
}

/// The good EPUB with one change made to it.
fn broken(change: impl FnOnce(&mut Vec<(String, bool, Vec<u8>)>)) -> Vec<Problem> {
    let mut files = epub::unzip(&ebook(&gulliver(), "classic")).unwrap();
    change(&mut files);
    epub::check(&rezip(&files))
}

fn edit(files: &mut [(String, bool, Vec<u8>)], path: &str, from: &str, to: &str) {
    let f = files.iter_mut().find(|f| f.0 == path).unwrap_or_else(|| panic!("no {path}"));
    let s = String::from_utf8(f.2.clone()).unwrap();
    assert!(s.contains(from), "{path} has no {from}");
    f.2 = s.replacen(from, to, 1).into_bytes();
}

fn has(problems: &[Problem], rule: Rule, words: &str) -> bool {
    problems.iter().any(|p| p.rule == rule && p.to_string().contains(words))
}

#[test]
fn each_check_finds_the_epub_broken_for_it() {
    let cases: Vec<(&str, Rule, &str, Box<dyn FnOnce(&mut Vec<(String, bool, Vec<u8>)>)>)> = vec![
        ("compressed mimetype", Rule::Container, "stored", Box::new(|f| f[0].1 = false)),
        ("wrong mimetype", Rule::Container, "application/epub+zip", Box::new(|f| f[0].2 = b"application/zip".to_vec())),
        ("mimetype not first", Rule::Container, "first file", Box::new(|f| f.swap(0, 1))),
        ("no container", Rule::Container, "missing", Box::new(|f| f.retain(|x| x.0 != "META-INF/container.xml"))),
        (
            "no title",
            Rule::Package,
            "no title",
            Box::new(|f| {
                edit(f, "EPUB/package.opf", "<dc:title>", "<dc:subject>");
                edit(f, "EPUB/package.opf", "</dc:title>", "</dc:subject>");
            }),
        ),
        ("bad modified date", Rule::Package, "dcterms:modified", Box::new(|f| edit(f, "EPUB/package.opf", "T00:00:00Z</meta>", " 00:00</meta>"))),
        ("a listed file missing", Rule::Manifest, "listed but not in", Box::new(|f| f.retain(|x| x.0 != "EPUB/text/s006.xhtml"))),
        ("an unlisted file", Rule::Manifest, "not in the manifest", Box::new(|f| f.push(("EPUB/extra.xhtml".into(), false, b"<x/>".to_vec())))),
        (
            "wrong media type",
            Rule::Manifest,
            "not text/css",
            Box::new(|f| edit(f, "EPUB/package.opf", "media-type=\"text/css\"", "media-type=\"text/plain\"")),
        ),
        (
            "spine names nothing",
            Rule::Spine,
            "not in the manifest",
            Box::new(|f| edit(f, "EPUB/package.opf", "<itemref idref=\"s006\"/>", "<itemref idref=\"zz\"/>")),
        ),
        (
            "spine twice",
            Rule::Spine,
            "twice",
            Box::new(|f| edit(f, "EPUB/package.opf", "<itemref idref=\"s006\"/>", "<itemref idref=\"s006\"/><itemref idref=\"s006\"/>")),
        ),
        ("no toc nav", Rule::Navigation, "one toc nav", Box::new(|f| edit(f, "EPUB/nav.xhtml", "epub:type=\"toc\"", "epub:type=\"other\""))),
        ("nav to nowhere", Rule::Links, "no id", Box::new(|f| edit(f, "EPUB/nav.xhtml", "#s006\"", "#nowhere\""))),
        ("ncx to nowhere", Rule::Links, "not in the EPUB", Box::new(|f| edit(f, "EPUB/toc.ncx", "text/s006.xhtml", "text/gone.xhtml"))),
        ("an id twice", Rule::Ids, "used twice", Box::new(|f| edit(f, "EPUB/text/s006.xhtml", "<header>", "<header id=\"s006\">"))),
        (
            "a broken link",
            Rule::Links,
            "goes nowhere",
            Box::new(|f| edit(f, "EPUB/text/s006.xhtml", "<header>", "<header><a href=\"s999.xhtml\">x</a>")),
        ),
        ("malformed XHTML", Rule::Xhtml, "not well-formed", Box::new(|f| edit(f, "EPUB/text/s006.xhtml", "</header>", "</head>"))),
        ("no language", Rule::Xhtml, "language", Box::new(|f| edit(f, "EPUB/text/s006.xhtml", " lang=\"en\"", ""))),
        ("positioning", Rule::Css, "position", Box::new(|f| edit(f, "EPUB/css/book.css", "body {", "body { position: absolute;"))),
        ("a float", Rule::Css, "float", Box::new(|f| edit(f, "EPUB/css/book.css", "body {", "body { float: left;"))),
        (
            "fixed type size",
            Rule::Css,
            "font-size in px",
            Box::new(|f| edit(f, "EPUB/css/book.css", "h1 { font-size: 1.6em;", "h1 { font-size: 22px;")),
        ),
        ("a font that is not there", Rule::Css, "goes nowhere", Box::new(|f| f.retain(|x| !x.0.ends_with("classic-serif-regular.ttf")))),
    ];
    for (name, rule, words, change) in cases {
        let problems = broken(change);
        assert!(has(&problems, rule, words), "{name}: expected a {} problem saying “{words}”, got {problems:#?}", rule.name());
    }
    assert!(has(&epub::check(b"not a zip"), Rule::Container, "not a zip"));
}

/// The EPUB's words: the core reader's and Book's agree, and the author's are the source's.
fn check_words(name: &str, b: &Book, bytes: &[u8]) {
    let ours = epub::words(bytes, false).unwrap();
    let core = epub::core_words(bytes).unwrap();
    if let Err(d) = verify::compare(&ours, &core) {
        panic!("{name}: the core's reader reads the EPUB differently: {d}");
    }
    let author = epub::words(bytes, true).unwrap();
    let source = verify::book_words(b, true);
    if let Err(d) = verify::compare(&source, &author) {
        panic!("{name}: the EPUB's words differ from the source's: {d}");
    }
}

#[test]
fn every_designs_epub_passes_and_keeps_every_word() {
    for (book_name, b) in [("gulliver", gulliver()), ("waverley", waverley())] {
        for d in builtin::all() {
            let name = format!("{book_name}-{}", d.id);
            let bytes = ebook(&b, &d.id);
            let problems = epub::check(&bytes);
            assert!(problems.is_empty(), "{name}: {problems:#?}");
            check_words(&name, &b, &bytes);
        }
    }
}

/// An EPUB's structure: package metadata, manifest, reading order, contents and landmarks, the
/// stylesheet, and each document's kind, heading, paragraphs and words.
fn structure(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let files = epub::unzip(bytes).unwrap();
    let get = |p: &str| String::from_utf8(files.iter().find(|f| f.0 == p).unwrap().2.clone()).unwrap();
    let opf_text = get("EPUB/package.opf");
    let opf = epub::parse(&opf_text).unwrap();
    let mut out = String::from("# package\n");
    for n in opf.descendants().filter(|n| n.is_element() && n.parent_element().is_some_and(|p| p.has_tag_name("metadata"))) {
        let key = n.attribute("property").map_or_else(|| n.tag_name().name().to_owned(), str::to_owned);
        let mut v = n.text().unwrap_or("").to_owned();
        if key == "identifier" {
            v = v.split(':').take(2).collect::<Vec<_>>().join(":") + ":…";
        }
        writeln!(out, "{key} = {v}").unwrap();
    }
    out += "# manifest\n";
    for n in opf.descendants().filter(|n| n.has_tag_name("item")) {
        let line = format!("{} {} {}", n.attribute("href").unwrap(), n.attribute("media-type").unwrap(), n.attribute("properties").unwrap_or(""));
        writeln!(out, "{}", line.trim_end()).unwrap();
    }
    out += "# spine\n";
    for n in opf.descendants().filter(|n| n.has_tag_name("itemref")) {
        writeln!(out, "{}{}", n.attribute("idref").unwrap(), if n.attribute("linear") == Some("no") { " (not linear)" } else { "" }).unwrap();
    }
    out += "# navigation\n";
    let nav_text = get("EPUB/nav.xhtml");
    let nav = epub::parse(&nav_text).unwrap();
    for a in nav.descendants().filter(|n| n.has_tag_name("a")) {
        let depth = a.ancestors().filter(|x| x.has_tag_name("ol")).count();
        let kind = a.attribute(("http://www.idpf.org/2007/ops", "type")).map(|t| format!("[{t}] ")).unwrap_or_default();
        writeln!(out, "{}{kind}{} -> {}", "  ".repeat(depth.saturating_sub(1)), a.text().unwrap_or(""), a.attribute("href").unwrap()).unwrap();
    }
    out += "# stylesheet\n";
    out += &get("EPUB/css/book.css");
    out += "# documents\n";
    for (path, _, b) in files.iter().filter(|f| f.0.starts_with("EPUB/text/")) {
        let t = String::from_utf8(b.clone()).unwrap();
        let d = epub::parse(&t).unwrap();
        let sec = d.descendants().find(|n| n.has_tag_name("section"));
        let kind = sec.and_then(|s| s.attribute(("http://www.idpf.org/2007/ops", "type"))).unwrap_or("-");
        let heading = d.descendants().find(|n| n.has_tag_name("h1")).and_then(|h| h.text()).unwrap_or("");
        let paras = d.descendants().filter(|n| n.has_tag_name("p")).count();
        let words: usize = d
            .descendants()
            .filter(|n| n.has_tag_name("p"))
            .map(|p| p.descendants().filter_map(|x| x.text()).collect::<String>().split_whitespace().count())
            .sum();
        writeln!(out, "{} {kind} “{heading}”: {paras} paragraphs, {words} words", path.trim_start_matches("EPUB/")).unwrap();
    }
    out
}

#[test]
fn every_designs_epub_matches_its_golden_structure() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/epub");
    let bless = std::env::var_os("BOOK_BLESS").is_some();
    let mut failures = Vec::new();
    for (book_name, b) in [("gulliver", gulliver()), ("waverley", waverley())] {
        for d in builtin::all() {
            let name = format!("{book_name}-{}", d.id);
            let got = structure(&ebook(&b, &d.id));
            let path = dir.join(format!("{name}.txt"));
            if bless {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(&path, &got).unwrap();
                continue;
            }
            let want = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{name}: no golden file; run with BOOK_BLESS=1"));
            if let Some(line) =
                got.lines().zip(want.lines()).position(|(a, b)| a != b).or((got != want).then(|| got.lines().count().min(want.lines().count())))
            {
                failures.push(format!(
                    "{name}: line {}: got `{}`, golden `{}`",
                    line + 1,
                    got.lines().nth(line).unwrap_or(""),
                    want.lines().nth(line).unwrap_or("")
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_standard_validator_finds_no_errors_where_it_is_installed() {
    let Some(jar) = std::env::var_os("BOOK_EPUB_VALIDATOR") else {
        eprintln!("the standard EPUB validator is not set up (BOOK_EPUB_VALIDATOR); skipped");
        return;
    };
    let java = std::env::var_os("BOOK_JAVA").unwrap_or_else(|| "java".into());
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/epub-validator");
    std::fs::create_dir_all(&dir).unwrap();
    let mut failures = Vec::new();
    for (book_name, b) in [("gulliver", gulliver()), ("waverley", waverley())] {
        for d in builtin::all() {
            let name = format!("{book_name}-{}", d.id);
            let path = dir.join(format!("{name}.epub"));
            std::fs::write(&path, ebook(&b, &d.id)).unwrap();
            let out = std::process::Command::new(&java).arg("-jar").arg(&jar).arg(&path).arg("-q").output().unwrap();
            let text = String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
            if !out.status.success() || ["ERROR", "FATAL", "WARNING"].iter().any(|w| text.contains(w)) {
                failures.push(format!("{name}:\n{text}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
