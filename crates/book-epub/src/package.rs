// SPDX-License-Identifier: MIT OR Apache-2.0
//! The EPUB 3 package: container, package document, navigation document, NCX and a
//! deterministic zip with `mimetype` first and stored.
//!
//! The package knows nothing of books: it takes finished files (XHTML, CSS, images, fonts), their
//! reading order, the table of contents and landmarks, and metadata, and writes the container
//! around them. The same package gives the same bytes every time.
//!
// core-candidate: C6, an EPUB 3 package layer any app that writes EPUB could use
// (core issue #20).

use std::io::{Cursor, Write as _};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

/// Where the package's own files live inside the zip.
pub const ROOT: &str = "EPUB";

/// One file of the publication, beside the package document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// Its manifest id (an XML name).
    pub id: String,
    /// Its path from the package document (`text/s001.xhtml`).
    pub href: String,
    /// Its media type.
    pub media_type: String,
    /// Its manifest properties (`cover-image`, `svg`, ...).
    pub properties: Vec<String>,
    /// Its bytes.
    pub bytes: Vec<u8>,
}

/// One entry of the table of contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavPoint {
    /// What the entry says.
    pub label: String,
    /// Where it goes, from the package document (`text/s003.xhtml#s3`).
    pub href: String,
    /// Entries under it.
    pub children: Vec<NavPoint>,
}

/// One landmark: a structural point a reading system can go to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landmark {
    /// Its `epub:type` (`cover`, `titlepage`, `toc`, `bodymatter`, ...).
    pub kind: String,
    /// What it says.
    pub label: String,
    /// Where it goes.
    pub href: String,
}

/// A publication's metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    /// The unique identifier (`urn:isbn:…` or `urn:uuid:…`).
    pub identifier: String,
    /// The title.
    pub title: String,
    /// The language (BCP 47).
    pub language: String,
    /// The creators, in order.
    pub creators: Vec<String>,
    /// When it was last changed, `YYYY-MM-DDThh:mm:ssZ`.
    pub modified: String,
    /// Other Dublin Core elements (`publisher`, `rights`, `date`, `description`, `subject`), in
    /// order, as (element, value).
    pub dc: Vec<(String, String)>,
    /// `meta` elements, as (property, value), in order (`schema:accessMode`, ...).
    pub meta: Vec<(String, String)>,
    /// `link` elements, as (rel, href), in order.
    pub links: Vec<(String, String)>,
}

/// A publication.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Package {
    /// Its metadata.
    pub metadata: Metadata,
    /// Its files, in manifest order (the navigation document and NCX are added by [`write`]).
    pub items: Vec<Item>,
    /// The reading order, by item id, with whether each is linear.
    pub spine: Vec<(String, bool)>,
    /// Where the navigation document sits in the reading order (an index into `spine`), when it
    /// is read as part of the book; otherwise it is outside the reading order.
    pub nav_in_spine: Option<usize>,
    /// The navigation document's heading ("Contents").
    pub nav_title: String,
    /// A stylesheet for the navigation document, by its href.
    pub nav_css: Option<String>,
    /// The table of contents.
    pub toc: Vec<NavPoint>,
    /// The landmarks.
    pub landmarks: Vec<Landmark>,
}

/// The navigation document's and NCX's manifest ids and paths.
pub const NAV: (&str, &str) = ("nav", "nav.xhtml");
/// The NCX's manifest id and path.
pub const NCX: (&str, &str) = ("ncx", "toc.ncx");

/// Text escaped for XML content and attribute values.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            // Characters XML 1.0 does not allow are left out.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            '\u{FFFE}' | '\u{FFFF}' => {}
            c => out.push(c),
        }
    }
    out
}

/// Why a package could not be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageError {
    /// Two items share an id or a path.
    Duplicate(String),
    /// The spine names an item that is not in the manifest.
    NotInManifest(String),
    /// Required metadata is missing (which).
    Missing(&'static str),
    /// The zip could not be written.
    Zip(String),
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageError::Duplicate(s) => write!(f, "two files of the ebook share “{s}”"),
            PackageError::NotInManifest(s) => write!(f, "the reading order names “{s}”, which is not in the ebook"),
            PackageError::Missing(s) => write!(f, "the ebook has no {s}"),
            PackageError::Zip(s) => write!(f, "the ebook could not be written: {s}"),
        }
    }
}

impl std::error::Error for PackageError {}

fn check(p: &Package) -> Result<(), PackageError> {
    let m = &p.metadata;
    for (empty, what) in [
        (m.identifier.is_empty(), "identifier"),
        (m.title.is_empty(), "title"),
        (m.language.is_empty(), "language"),
        (m.modified.is_empty(), "modified date"),
    ] {
        if empty {
            return Err(PackageError::Missing(what));
        }
    }
    let mut ids = std::collections::BTreeSet::from([NAV.0, NCX.0]);
    let mut hrefs = std::collections::BTreeSet::from([NAV.1, NCX.1]);
    for i in &p.items {
        if !ids.insert(&i.id) {
            return Err(PackageError::Duplicate(i.id.clone()));
        }
        if !hrefs.insert(&i.href) {
            return Err(PackageError::Duplicate(i.href.clone()));
        }
    }
    for (id, _) in &p.spine {
        if !p.items.iter().any(|i| &i.id == id) {
            return Err(PackageError::NotInManifest(id.clone()));
        }
    }
    Ok(())
}

/// The container file (`META-INF/container.xml`).
pub fn container() -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n  <rootfiles>\n    <rootfile full-path=\"{ROOT}/package.opf\" media-type=\"application/oebps-package+xml\"/>\n  </rootfiles>\n</container>\n"
    )
}

/// The package document (`package.opf`).
pub fn opf(p: &Package) -> String {
    let m = &p.metadata;
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s += &format!(
        "<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uid\" xml:lang=\"{}\" prefix=\"schema: http://schema.org/ a11y: http://www.idpf.org/epub/vocab/package/a11y/#\">\n",
        escape(&m.language)
    );
    s += "  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n";
    s += &format!("    <dc:identifier id=\"uid\">{}</dc:identifier>\n", escape(&m.identifier));
    s += &format!("    <dc:title>{}</dc:title>\n", escape(&m.title));
    s += &format!("    <dc:language>{}</dc:language>\n", escape(&m.language));
    for (i, c) in m.creators.iter().enumerate() {
        s += &format!("    <dc:creator id=\"creator{}\">{}</dc:creator>\n", i + 1, escape(c));
    }
    for (el, v) in &m.dc {
        s += &format!("    <dc:{el}>{}</dc:{el}>\n", escape(v));
    }
    s += &format!("    <meta property=\"dcterms:modified\">{}</meta>\n", escape(&m.modified));
    for (prop, v) in &m.meta {
        s += &format!("    <meta property=\"{}\">{}</meta>\n", escape(prop), escape(v));
    }
    for (rel, href) in &m.links {
        s += &format!("    <link rel=\"{}\" href=\"{}\"/>\n", escape(rel), escape(href));
    }
    s += "  </metadata>\n  <manifest>\n";
    s += &format!("    <item id=\"{}\" href=\"{}\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n", NAV.0, NAV.1);
    s += &format!("    <item id=\"{}\" href=\"{}\" media-type=\"application/x-dtbncx+xml\"/>\n", NCX.0, NCX.1);
    for i in &p.items {
        let props = if i.properties.is_empty() { String::new() } else { format!(" properties=\"{}\"", escape(&i.properties.join(" "))) };
        s += &format!("    <item id=\"{}\" href=\"{}\" media-type=\"{}\"{props}/>\n", escape(&i.id), escape(&i.href), escape(&i.media_type));
    }
    s += &format!("  </manifest>\n  <spine toc=\"{}\">\n", NCX.0);
    let mut spine: Vec<(&str, bool)> = p.spine.iter().map(|(id, l)| (id.as_str(), *l)).collect();
    if let Some(at) = p.nav_in_spine {
        spine.insert(at.min(spine.len()), (NAV.0, true));
    }
    for (id, linear) in spine {
        let l = if linear { "" } else { " linear=\"no\"" };
        s += &format!("    <itemref idref=\"{}\"{l}/>\n", escape(id));
    }
    s += "  </spine>\n</package>\n";
    s
}

fn nav_list(points: &[NavPoint], depth: usize, s: &mut String) {
    let pad = "  ".repeat(depth + 3);
    *s += &format!("{pad}<ol>\n");
    for p in points {
        *s += &format!("{pad}  <li><a href=\"{}\">{}</a>", escape(&p.href), escape(&p.label));
        if p.children.is_empty() {
            *s += "</li>\n";
        } else {
            *s += "\n";
            nav_list(&p.children, depth + 2, s);
            *s += &format!("{pad}  </li>\n");
        }
    }
    *s += &format!("{pad}</ol>\n");
}

/// The navigation document (`nav.xhtml`): the `toc` nav and the `landmarks` nav.
pub fn nav(p: &Package) -> String {
    let lang = escape(&p.metadata.language);
    let title = if p.nav_title.is_empty() { "Contents".to_owned() } else { escape(&p.nav_title) };
    let mut s = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" lang=\"{lang}\" xml:lang=\"{lang}\">\n  <head>\n    <meta charset=\"UTF-8\"/>\n    <title>{title}</title>\n"
    );
    if let Some(css) = &p.nav_css {
        s += &format!("    <link rel=\"stylesheet\" type=\"text/css\" href=\"{}\"/>\n", escape(css));
    }
    s += "  </head>\n  <body epub:type=\"frontmatter\">\n";
    s += &format!("    <nav epub:type=\"toc\" id=\"toc\" role=\"doc-toc\">\n      <h1>{title}</h1>\n");
    nav_list(&p.toc, 0, &mut s);
    s += "    </nav>\n";
    if !p.landmarks.is_empty() {
        s += "    <nav epub:type=\"landmarks\" id=\"landmarks\" hidden=\"hidden\">\n      <h2>Landmarks</h2>\n      <ol>\n";
        for l in &p.landmarks {
            s += &format!("        <li><a epub:type=\"{}\" href=\"{}\">{}</a></li>\n", escape(&l.kind), escape(&l.href), escape(&l.label));
        }
        s += "      </ol>\n    </nav>\n";
    }
    s += "  </body>\n</html>\n";
    s
}

fn ncx_points(points: &[NavPoint], order: &mut usize, depth: usize, s: &mut String) {
    let pad = "  ".repeat(depth + 2);
    for p in points {
        *order += 1;
        *s += &format!(
            "{pad}<navPoint id=\"np{0}\" playOrder=\"{0}\">\n{pad}  <navLabel><text>{1}</text></navLabel>\n{pad}  <content src=\"{2}\"/>\n",
            order,
            escape(&p.label),
            escape(&p.href)
        );
        ncx_points(&p.children, order, depth + 1, s);
        *s += &format!("{pad}</navPoint>\n");
    }
}

fn depth(points: &[NavPoint]) -> usize {
    points.iter().map(|p| 1 + depth(&p.children)).max().unwrap_or(0)
}

/// The NCX (`toc.ncx`), for older reading systems: the same entries as the `toc` nav.
pub fn ncx(p: &Package) -> String {
    let m = &p.metadata;
    let mut s = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s += &format!("<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\" xml:lang=\"{}\">\n", escape(&m.language));
    s += &format!(
        "  <head>\n    <meta name=\"dtb:uid\" content=\"{}\"/>\n    <meta name=\"dtb:depth\" content=\"{}\"/>\n    <meta name=\"dtb:totalPageCount\" content=\"0\"/>\n    <meta name=\"dtb:maxPageNumber\" content=\"0\"/>\n  </head>\n",
        escape(&m.identifier),
        depth(&p.toc).max(1)
    );
    s += &format!("  <docTitle><text>{}</text></docTitle>\n  <navMap>\n", escape(&m.title));
    let mut order = 0;
    ncx_points(&p.toc, &mut order, 0, &mut s);
    s += "  </navMap>\n</ncx>\n";
    s
}

/// Every file of the zip, in order, as (path, bytes).
pub fn files(p: &Package) -> Vec<(String, Vec<u8>)> {
    let mut out = vec![
        ("mimetype".to_owned(), b"application/epub+zip".to_vec()),
        ("META-INF/container.xml".to_owned(), container().into_bytes()),
        (format!("{ROOT}/package.opf"), opf(p).into_bytes()),
        (format!("{ROOT}/{}", NAV.1), nav(p).into_bytes()),
        (format!("{ROOT}/{}", NCX.1), ncx(p).into_bytes()),
    ];
    out.extend(p.items.iter().map(|i| (format!("{ROOT}/{}", i.href), i.bytes.clone())));
    out
}

/// The EPUB: `mimetype` first and stored, every other file compressed, every date the same, so
/// the same package gives the same bytes.
pub fn write(p: &Package) -> Result<Vec<u8>, PackageError> {
    check(p)?;
    let z = |e: zip::result::ZipError| PackageError::Zip(e.to_string());
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let stored =
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored).last_modified_time(DateTime::default()).unix_permissions(0o644);
    let deflated = stored.compression_method(CompressionMethod::Deflated);
    for (path, bytes) in files(p) {
        let options = if path == "mimetype" { stored } else { deflated };
        zip.start_file(path, options).map_err(z)?;
        zip.write_all(&bytes).map_err(|e| PackageError::Zip(e.to_string()))?;
    }
    Ok(zip.finish().map_err(z)?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> Package {
        Package {
            metadata: Metadata {
                identifier: "urn:uuid:0".into(),
                title: "A & B".into(),
                language: "en".into(),
                creators: vec!["Someone".into()],
                modified: "2026-11-24T00:00:00Z".into(),
                ..Metadata::default()
            },
            items: vec![Item {
                id: "a".into(),
                href: "a.xhtml".into(),
                media_type: "application/xhtml+xml".into(),
                properties: vec![],
                bytes: b"<x/>".to_vec(),
            }],
            spine: vec![("a".into(), true)],
            toc: vec![NavPoint { label: "One".into(), href: "a.xhtml".into(), children: vec![] }],
            ..Package::default()
        }
    }

    #[test]
    fn escape_keeps_xml_well_formed() {
        assert_eq!(escape("a<b>&\"c'\u{1}"), "a&lt;b&gt;&amp;&quot;c&#39;");
    }

    #[test]
    fn a_package_writes_the_same_bytes_with_mimetype_first() {
        let p = small();
        let a = write(&p).unwrap();
        assert_eq!(a, write(&p).unwrap());
        assert_eq!(&a[30..38], b"mimetype");
        assert!(opf(&p).contains("<dc:title>A &amp; B</dc:title>"));
        assert!(nav(&p).contains("epub:type=\"toc\"") && ncx(&p).contains("playOrder=\"1\""));
    }

    #[test]
    fn bad_packages_are_refused() {
        let mut p = small();
        p.spine.push(("missing".into(), true));
        assert_eq!(write(&p), Err(PackageError::NotInManifest("missing".into())));
        let mut p = small();
        p.items.push(p.items[0].clone());
        assert!(matches!(write(&p), Err(PackageError::Duplicate(_))));
        let mut p = small();
        p.metadata.title.clear();
        assert_eq!(write(&p), Err(PackageError::Missing("title")));
    }
}
