// SPDX-License-Identifier: MIT OR Apache-2.0
//! Book's own EPUB checks (plan M6.3): the container, the package document, the manifest and
//! spine, the navigation document, ids, links, well-formed XHTML and CSS within the subset Book
//! writes (SPEC 6.8); and the words of an EPUB, read with Book's own reader and with the core's,
//! for the word-for-word check.
//!
//! These run on any EPUB, not only Book's, and never trust it: the number of files, their sizes
//! and the total read are bounded, and a broken file gives problems, never a panic.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read as _;

use crate::verify;

/// Most files read from an EPUB.
pub const MAX_FILES: usize = 10_000;
/// Largest single file read, in bytes.
pub const MAX_FILE: u64 = 64 << 20;
/// Most bytes read from an EPUB in all.
pub const MAX_TOTAL: u64 = 1 << 30;

/// Which rule a problem breaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// The zip and its `mimetype` file.
    Container,
    /// The package document's metadata.
    Package,
    /// The manifest: every file listed and present, media types.
    Manifest,
    /// The reading order.
    Spine,
    /// The navigation document and NCX.
    Navigation,
    /// Ids unique within each document.
    Ids,
    /// Links and references that go nowhere.
    Links,
    /// XHTML that is not well-formed or lacks its language.
    Xhtml,
    /// CSS outside the conservative subset.
    Css,
}

impl Rule {
    /// The rule's name in reports.
    pub fn name(self) -> &'static str {
        match self {
            Rule::Container => "container",
            Rule::Package => "package",
            Rule::Manifest => "manifest",
            Rule::Spine => "spine",
            Rule::Navigation => "navigation",
            Rule::Ids => "ids",
            Rule::Links => "links",
            Rule::Xhtml => "xhtml",
            Rule::Css => "css",
        }
    }
}

/// One problem found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// The rule it breaks.
    pub rule: Rule,
    /// The file it is in (empty for the whole EPUB).
    pub file: String,
    /// What is wrong, in words.
    pub message: String,
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.file.is_empty() {
            write!(f, "{}: {}", self.rule.name(), self.message)
        } else {
            write!(f, "{}: {}: {}", self.rule.name(), self.file, self.message)
        }
    }
}

/// An EPUB's files, in zip order: (path, stored uncompressed, bytes).
pub type Files = Vec<(String, bool, Vec<u8>)>;

/// The files of an EPUB, bounded.
pub fn unzip(bytes: &[u8]) -> Result<Files, String> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("the file is not a zip: {e}"))?;
    if z.len() > MAX_FILES {
        return Err(format!("the EPUB has more than {MAX_FILES} files"));
    }
    let mut total = 0u64;
    let mut out = Vec::with_capacity(z.len());
    for i in 0..z.len() {
        let f = z.by_index(i).map_err(|e| format!("file {} cannot be read: {e}", i + 1))?;
        let name = f.name().to_owned();
        let stored = f.compression() == zip::CompressionMethod::Stored;
        let mut b = Vec::new();
        f.take(MAX_FILE + 1).read_to_end(&mut b).map_err(|e| format!("{name} cannot be read: {e}"))?;
        if b.len() as u64 > MAX_FILE {
            return Err(format!("{name} is larger than {} MB", MAX_FILE >> 20));
        }
        total += b.len() as u64;
        if total > MAX_TOTAL {
            return Err(format!("the EPUB holds more than {} MB", MAX_TOTAL >> 20));
        }
        out.push((name, stored, b));
    }
    Ok(out)
}

/// An XML file, allowing the `<!DOCTYPE html>` XHTML carries (but no entities).
pub fn parse(text: &str) -> Result<roxmltree::Document<'_>, roxmltree::Error> {
    roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: true, nodes_limit: 4_000_000, ..Default::default() })
}

/// A path from `base` (a file) to `href`, with `..` and `.` resolved; `None` if it leaves the
/// zip or is absolute.
pub fn resolve(base: &str, href: &str) -> Option<String> {
    if href.starts_with('/') {
        return None;
    }
    let dir = base.rsplit_once('/').map_or("", |(d, _)| d);
    let mut parts: Vec<&str> = if dir.is_empty() { Vec::new() } else { dir.split('/').collect() };
    for seg in href.split('/') {
        match seg {
            ".." => {
                parts.pop()?;
            }
            "." | "" => {}
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}

const OPS: &str = "http://www.idpf.org/2007/ops";
const XHTML: &str = "http://www.w3.org/1999/xhtml";
const XML: &str = "http://www.w3.org/XML/1998/namespace";

struct Checker {
    files: BTreeMap<String, Vec<u8>>,
    problems: Vec<Problem>,
}

impl Checker {
    fn add(&mut self, rule: Rule, file: &str, message: impl Into<String>) {
        if self.problems.len() < 1000 {
            self.problems.push(Problem { rule, file: file.to_owned(), message: message.into() });
        }
    }

    fn text(&self, path: &str) -> Option<String> {
        self.files.get(path).and_then(|b| String::from_utf8(b.clone()).ok())
    }

    /// Whether `href` from `from` reaches a file (and an id in it, if it names one).
    fn target_exists(&self, from: &str, href: &str) -> Result<(), String> {
        let (file, frag) = href.split_once('#').unwrap_or((href, ""));
        let path = if file.is_empty() { from.to_owned() } else { resolve(from, &percent_decode(file)).ok_or("it leaves the EPUB")? };
        if !self.files.contains_key(&path) {
            return Err(format!("{path} is not in the EPUB"));
        }
        if frag.is_empty() {
            return Ok(());
        }
        let body = self.text(&path).ok_or_else(|| format!("{path} is not a document"))?;
        if !body.contains(&format!("id=\"{frag}\"")) {
            let doc = parse(&body).map_err(|_| format!("{path} cannot be read"))?;
            if !doc.descendants().any(|n| n.attribute("id") == Some(frag)) {
                return Err(format!("{path} has no id “{frag}”"));
            }
        }
        Ok(())
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A manifest item.
struct Item {
    id: String,
    path: String,
    media: String,
    properties: Vec<String>,
}

/// Every problem Book finds in an EPUB, in the order found.
pub fn check(bytes: &[u8]) -> Vec<Problem> {
    let files = match unzip(bytes) {
        Ok(f) => f,
        Err(e) => return vec![Problem { rule: Rule::Container, file: String::new(), message: e }],
    };
    let mut c = Checker { files: files.iter().map(|(n, _, b)| (n.clone(), b.clone())).collect(), problems: Vec::new() };
    match files.first() {
        Some((n, stored, b)) if n == "mimetype" => {
            if !stored {
                c.add(Rule::Container, "mimetype", "it is compressed; it must be stored");
            }
            if b != b"application/epub+zip" {
                c.add(Rule::Container, "mimetype", "it must say application/epub+zip and nothing else");
            }
        }
        _ => c.add(Rule::Container, "", "the first file is not mimetype"),
    }
    let Some(opf_path) = container(&mut c) else { return c.problems };
    let Some(opf_text) = c.text(&opf_path) else {
        c.add(Rule::Container, "META-INF/container.xml", format!("{opf_path} is not in the EPUB"));
        return c.problems;
    };
    let opf = match parse(&opf_text) {
        Ok(d) => d,
        Err(e) => {
            c.add(Rule::Package, &opf_path, format!("not well-formed: {e}"));
            return c.problems;
        }
    };
    package_metadata(&mut c, &opf_path, &opf);
    let items = manifest(&mut c, &opf_path, &opf);
    spine(&mut c, &opf_path, &opf, &items);
    for item in &items {
        match item.media.as_str() {
            "application/xhtml+xml" => xhtml(&mut c, item),
            "text/css" => css(&mut c, &item.path),
            _ => {}
        }
    }
    navigation(&mut c, &items);
    c.problems
}

fn container(c: &mut Checker) -> Option<String> {
    let path = "META-INF/container.xml";
    let Some(text) = c.text(path) else {
        c.add(Rule::Container, path, "missing");
        return None;
    };
    let doc = match parse(&text) {
        Ok(d) => d,
        Err(e) => {
            c.add(Rule::Container, path, format!("not well-formed: {e}"));
            return None;
        }
    };
    let root = doc.descendants().find(|n| n.has_tag_name("rootfile") && n.attribute("media-type") == Some("application/oebps-package+xml"));
    let full = root.and_then(|n| n.attribute("full-path")).map(str::to_owned);
    if full.is_none() {
        c.add(Rule::Container, path, "it names no package document");
    }
    full
}

fn package_metadata(c: &mut Checker, path: &str, opf: &roxmltree::Document) {
    let root = opf.root_element();
    if root.attribute("version") != Some("3.0") {
        c.add(Rule::Package, path, "the package is not EPUB 3 (version 3.0)");
    }
    let uid = root.attribute("unique-identifier").unwrap_or("");
    let has = |name: &str| opf.descendants().any(|n| n.tag_name().name() == name && n.text().is_some_and(|t| !t.trim().is_empty()));
    if !opf
        .descendants()
        .any(|n| n.tag_name().name() == "identifier" && n.attribute("id") == Some(uid) && n.text().is_some_and(|t| !t.trim().is_empty()))
    {
        c.add(Rule::Package, path, "the unique identifier is missing or empty");
    }
    for (name, what) in [("title", "title"), ("language", "language")] {
        if !has(name) {
            c.add(Rule::Package, path, format!("no {what}"));
        }
    }
    let modified: Vec<&str> = opf.descendants().filter(|n| n.attribute("property") == Some("dcterms:modified")).filter_map(|n| n.text()).collect();
    let ok = modified.len() == 1 && {
        let m = modified[0].trim().as_bytes();
        m.len() == 20 && m[4] == b'-' && m[7] == b'-' && m[10] == b'T' && m[13] == b':' && m[16] == b':' && m[19] == b'Z'
    };
    if !ok {
        c.add(Rule::Package, path, "dcterms:modified must appear once, as YYYY-MM-DDThh:mm:ssZ");
    }
}

/// The media type a file's extension calls for, for the types Book checks.
fn media_for(path: &str) -> Option<&'static str> {
    let ext = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())?;
    Some(match ext.as_str() {
        "xhtml" => "application/xhtml+xml",
        "css" => "text/css",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ncx" => "application/x-dtbncx+xml",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => return None,
    })
}

fn manifest(c: &mut Checker, opf_path: &str, opf: &roxmltree::Document) -> Vec<Item> {
    let mut items = Vec::new();
    let mut ids = BTreeSet::new();
    for n in opf.descendants().filter(|n| n.has_tag_name("item")) {
        let (Some(id), Some(href), Some(media)) = (n.attribute("id"), n.attribute("href"), n.attribute("media-type")) else {
            c.add(Rule::Manifest, opf_path, "an item lacks its id, href or media type");
            continue;
        };
        if !ids.insert(id.to_owned()) {
            c.add(Rule::Ids, opf_path, format!("the id “{id}” is used twice"));
        }
        let Some(path) = resolve(opf_path, &percent_decode(href)) else {
            c.add(Rule::Manifest, opf_path, format!("{href} leaves the EPUB"));
            continue;
        };
        if !c.files.contains_key(&path) {
            c.add(Rule::Manifest, opf_path, format!("{path} is listed but not in the EPUB"));
        }
        if let Some(want) = media_for(&path)
            && want != media
            && !(want == "font/ttf" && media == "application/font-sfnt")
        {
            c.add(Rule::Manifest, opf_path, format!("{path} is listed as {media}, not {want}"));
        }
        let properties = n.attribute("properties").unwrap_or("").split_whitespace().map(str::to_owned).collect();
        items.push(Item { id: id.to_owned(), path, media: media.to_owned(), properties });
    }
    let navs = items.iter().filter(|i| i.properties.iter().any(|p| p == "nav")).count();
    if navs != 1 {
        c.add(Rule::Navigation, opf_path, format!("{navs} navigation documents; there must be one"));
    }
    if items.iter().filter(|i| i.properties.iter().any(|p| p == "cover-image")).count() > 1 {
        c.add(Rule::Manifest, opf_path, "more than one cover image");
    }
    let listed: BTreeSet<&str> = items.iter().map(|i| i.path.as_str()).collect();
    let unlisted: Vec<String> = c
        .files
        .keys()
        .filter(|f| *f != "mimetype" && !f.starts_with("META-INF/") && *f != opf_path && !f.ends_with('/') && !listed.contains(f.as_str()))
        .cloned()
        .collect();
    for f in unlisted {
        c.add(Rule::Manifest, &f, "in the EPUB but not in the manifest");
    }
    items
}

fn spine(c: &mut Checker, opf_path: &str, opf: &roxmltree::Document, items: &[Item]) {
    let refs: Vec<&str> = opf.descendants().filter(|n| n.has_tag_name("itemref")).filter_map(|n| n.attribute("idref")).collect();
    if refs.is_empty() {
        c.add(Rule::Spine, opf_path, "the reading order is empty");
    }
    let mut seen = BTreeSet::new();
    for r in refs {
        match items.iter().find(|i| i.id == r) {
            None => c.add(Rule::Spine, opf_path, format!("the reading order names “{r}”, which is not in the manifest")),
            Some(i) if i.media != "application/xhtml+xml" && i.media != "image/svg+xml" => {
                c.add(Rule::Spine, opf_path, format!("“{r}” in the reading order is {}, not a document", i.media));
            }
            Some(_) => {}
        }
        if !seen.insert(r) {
            c.add(Rule::Spine, opf_path, format!("“{r}” is in the reading order twice"));
        }
    }
}

fn xhtml(c: &mut Checker, item: &Item) {
    let path = &item.path;
    let Some(text) = c.text(path) else { return };
    let doc = match parse(&text) {
        Ok(d) => d,
        Err(e) => {
            c.add(Rule::Xhtml, path, format!("not well-formed: {e}"));
            return;
        }
    };
    let root = doc.root_element();
    if root.tag_name().name() != "html" || root.tag_name().namespace() != Some(XHTML) {
        c.add(Rule::Xhtml, path, "the root is not an XHTML html element");
    }
    let lang = |ns: Option<&str>| root.attributes().any(|a| a.name() == "lang" && a.namespace() == ns && !a.value().is_empty());
    if !lang(None) || !lang(Some(XML)) {
        c.add(Rule::Xhtml, path, "the document does not give its language (lang and xml:lang)");
    }
    let mut ids = BTreeSet::new();
    let mut problems = Vec::new();
    for n in doc.descendants().filter(|n| n.is_element()) {
        if let Some(id) = n.attribute("id")
            && !ids.insert(id)
        {
            problems.push((Rule::Ids, format!("the id “{id}” is used twice")));
        }
        let link = match n.tag_name().name() {
            "a" | "link" => n.attribute("href"),
            "img" | "script" | "audio" | "video" | "source" => n.attribute("src"),
            _ => None,
        };
        if let Some(href) = link
            && !href.contains(':')
            && let Err(e) = c.target_exists(path, href)
        {
            problems.push((Rule::Links, format!("{href} goes nowhere: {e}")));
        }
        if n.tag_name().name() == "img" && n.attribute("alt").is_none() {
            problems.push((Rule::Xhtml, "a picture has no alt attribute".into()));
        }
    }
    for (rule, m) in problems {
        c.add(rule, path, m);
    }
}

/// CSS within the subset Book writes (SPEC 6.8): no positioning, no floats but a drop cap, no
/// fixed sizes for text, no imports from outside, and every `url()` inside the EPUB.
fn css(c: &mut Checker, path: &str) {
    let Some(text) = c.text(path) else { return };
    let mut problems = Vec::new();
    // Comments out, then rule by rule.
    let mut clean = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(i) = rest.find("/*") {
        clean.push_str(&rest[..i]);
        rest = rest[i + 2..].split_once("*/").map_or("", |(_, r)| r);
    }
    clean.push_str(rest);
    for rule in clean.split('}') {
        let Some((selector, body)) = rule.split_once('{') else { continue };
        let selector = selector.trim();
        for decl in body.split(';') {
            let Some((prop, value)) = decl.split_once(':') else { continue };
            let raw = value.trim();
            let (prop, value) = (prop.trim().to_ascii_lowercase(), raw.to_ascii_lowercase());
            if prop == "position" && matches!(value.as_str(), "absolute" | "fixed") {
                problems.push(format!("{selector}: position: {value}"));
            }
            if prop == "float" && value != "none" && !selector.contains(".drop") {
                problems.push(format!("{selector}: float is only for drop caps"));
            }
            if prop == "font-size" && (value.ends_with("px") || value.ends_with("pt")) {
                problems.push(format!("{selector}: font-size in {}; use em", &value[value.len() - 2..]));
            }
            if let Some(i) = value.find("url(") {
                let inner = raw[i + 4..].split(')').next().unwrap_or("").trim_matches(|ch| ch == '"' || ch == '\'');
                if inner.contains(':') {
                    problems.push(format!("{selector}: {inner} is outside the EPUB"));
                } else if let Err(e) = c.target_exists(path, inner) {
                    problems.push(format!("{selector}: {inner} goes nowhere: {e}"));
                }
            }
        }
        if selector.starts_with("@import") {
            problems.push("@import is not used".into());
        }
    }
    for m in problems {
        c.add(Rule::Css, path, m);
    }
}

fn navigation(c: &mut Checker, items: &[Item]) {
    let Some(nav) = items.iter().find(|i| i.properties.iter().any(|p| p == "nav")) else { return };
    let Some(text) = c.text(&nav.path) else { return };
    let Ok(doc) = parse(&text) else { return };
    let tocs: Vec<_> = doc.descendants().filter(|n| n.has_tag_name("nav") && n.attribute((OPS, "type")) == Some("toc")).collect();
    if tocs.len() != 1 {
        c.add(Rule::Navigation, &nav.path, "there must be one toc nav");
    } else if !tocs[0].descendants().any(|n| n.has_tag_name("a")) {
        c.add(Rule::Navigation, &nav.path, "the toc nav is empty");
    }
    if let Some(ncx) = items.iter().find(|i| i.media == "application/x-dtbncx+xml") {
        let Some(t) = c.text(&ncx.path) else { return };
        match parse(&t) {
            Err(e) => c.add(Rule::Navigation, &ncx.path, format!("not well-formed: {e}")),
            Ok(d) => {
                let bad: Vec<String> = d
                    .descendants()
                    .filter(|n| n.has_tag_name("content"))
                    .filter_map(|n| n.attribute("src"))
                    .filter_map(|src| c.target_exists(&ncx.path, src).err().map(|e| format!("{src} goes nowhere: {e}")))
                    .collect();
                for m in bad {
                    c.add(Rule::Links, &ncx.path, m);
                }
            }
        }
    }
}

/// The words of an EPUB's reading order as Book's own reader finds them, block by block. With
/// `author`, only the author's words: what Book wrote (the contents, generated pages, chapter
/// number labels, note numbers and back links, the notes heading, scene-break marks, Book's own
/// subtitles and epigraphs, front and back matter titles) is left out.
pub fn words(bytes: &[u8], author: bool) -> Result<Vec<String>, String> {
    let files = unzip(bytes)?;
    let map: BTreeMap<&str, &[u8]> = files.iter().map(|(n, _, b)| (n.as_str(), b.as_slice())).collect();
    let text = |p: &str| map.get(p).and_then(|b| std::str::from_utf8(b).ok()).ok_or_else(|| format!("{p} is missing"));
    let cont = parse(text("META-INF/container.xml")?).map_err(|e| e.to_string())?;
    let opf_path = cont.descendants().find_map(|n| n.attribute("full-path")).ok_or("no package document")?.to_owned();
    let opf_text = text(&opf_path)?;
    let opf = parse(opf_text).map_err(|e| e.to_string())?;
    let hrefs: BTreeMap<&str, (&str, bool)> = opf
        .descendants()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|n| Some((n.attribute("id")?, (n.attribute("href")?, n.attribute("properties").is_some_and(|p| p.contains("nav"))))))
        .collect();
    let mut out = Vec::new();
    for idref in opf.descendants().filter(|n| n.has_tag_name("itemref")).filter_map(|n| n.attribute("idref")) {
        let Some((href, is_nav)) = hrefs.get(idref) else { continue };
        if author && *is_nav {
            continue;
        }
        let path = resolve(&opf_path, href).ok_or("a path leaves the EPUB")?;
        let doc = parse(text(&path)?).map_err(|e| format!("{path}: {e}"))?;
        for n in doc.descendants().filter(|n| is_block(n)) {
            if author && generated(&n) {
                continue;
            }
            let mut s = String::new();
            collect(n, author, &mut s);
            out.extend(verify::words(&s));
        }
    }
    Ok(out)
}

/// A block whose text is one paragraph: `p`, headings, and contents entries.
fn is_block(n: &roxmltree::Node) -> bool {
    n.is_element()
        && (matches!(n.tag_name().name(), "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
            || (n.has_tag_name("a") && n.parent_element().is_some_and(|p| p.has_tag_name("li"))))
}

fn has_class(n: &roxmltree::Node, class: &str) -> bool {
    n.attribute("class").is_some_and(|c| c.split_whitespace().any(|x| x == class))
}

/// Whether a block is Book's text, not the author's.
fn generated(n: &roxmltree::Node) -> bool {
    if ["label", "break", "subtitle"].iter().any(|c| has_class(n, c)) {
        return true;
    }
    if n.ancestors().any(|a| has_class(&a, "generated") || has_class(&a, "epigraph") || a.attribute((OPS, "type")) == Some("cover")) {
        return true;
    }
    if n.has_tag_name("h2") && n.ancestors().any(|a| has_class(&a, "notes")) {
        return true;
    }
    // A section's title is the author's for parts and chapters only.
    if n.has_tag_name("h1")
        && let Some(sec) = n.ancestors().find(|a| a.has_tag_name("section"))
    {
        return !matches!(sec.attribute((OPS, "type")), Some("chapter" | "part"));
    }
    false
}

fn collect(n: roxmltree::Node, author: bool, out: &mut String) {
    for ch in n.children() {
        if ch.is_text() {
            out.push_str(ch.text().unwrap_or(""));
        } else if ch.is_element() {
            if author && (has_class(&ch, "noteref") || ch.attribute("role") == Some("doc-backlink")) {
                continue;
            }
            if ch.has_tag_name("br") {
                out.push('\n');
            }
            collect(ch, author, out);
        }
    }
}

/// The words of an EPUB read back through the core's reader (`jcj-interchange`).
pub fn core_words(bytes: &[u8]) -> Result<Vec<String>, String> {
    let doc = jcj_interchange::import(jcj_interchange::Format::Epub, bytes).map_err(|e| e.to_string())?;
    Ok(verify::document_words(&doc))
}
