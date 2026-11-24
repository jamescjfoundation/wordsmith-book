// SPDX-License-Identifier: MIT OR Apache-2.0
//! The ebook package (plan M6.1, EBK-1, EBK-2): every fixture's EPUB has `mimetype` first and
//! stored, a container pointing at the package document, a manifest whose files are all there,
//! a spine of XHTML documents, a navigation document and NCX whose every target exists, and
//! well-formed XML throughout; two builds give the same bytes.

mod common;

use std::collections::BTreeMap;
use std::io::Read as _;

use common::*;
use wordsmith_book_check::build;
use wordsmith_book_design::builtin;
use wordsmith_book_import::Format;
use wordsmith_book_model::Edition;
use wordsmith_manuscript_format::book::{self, Book};

/// The zip's files, in order, with whether each was stored uncompressed.
fn unzip(bytes: &[u8]) -> Vec<(String, bool, Vec<u8>)> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    (0..z.len())
        .map(|i| {
            let mut f = z.by_index(i).unwrap();
            let mut b = Vec::new();
            f.read_to_end(&mut b).unwrap();
            (f.name().to_owned(), f.compression() == zip::CompressionMethod::Stored, b)
        })
        .collect()
}

/// An XML file, allowing the `<!DOCTYPE html>` XHTML carries.
fn parse(text: &str) -> Result<roxmltree::Document<'_>, roxmltree::Error> {
    roxmltree::Document::parse_with_options(text, roxmltree::ParsingOptions { allow_dtd: true, ..Default::default() })
}

/// Every id in an XHTML file.
fn ids(doc: &roxmltree::Document) -> Vec<String> {
    doc.descendants().filter_map(|n| n.attribute("id").map(str::to_owned)).collect()
}

fn resolve(base: &str, href: &str) -> String {
    let dir = base.rsplit_once('/').map_or("", |(d, _)| d);
    let mut parts: Vec<&str> = if dir.is_empty() { Vec::new() } else { dir.split('/').collect() };
    for seg in href.split('/') {
        match seg {
            ".." => {
                parts.pop();
            }
            "." | "" => {}
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// The structure of one EPUB; gives how many documents and navigation targets it has.
fn check_structure(name: &str, bytes: &[u8]) -> (usize, usize) {
    let files = unzip(bytes);
    assert_eq!(files[0].0, "mimetype", "{name}");
    assert!(files[0].1, "{name}: mimetype is stored");
    assert_eq!(files[0].2, b"application/epub+zip");
    let map: BTreeMap<String, Vec<u8>> = files.iter().map(|(n, _, b)| (n.clone(), b.clone())).collect();
    let text = |p: &str| String::from_utf8(map.get(p).unwrap_or_else(|| panic!("{name}: no {p}")).clone()).unwrap();
    let container = text("META-INF/container.xml");
    let c = parse(&container).unwrap();
    let opf_path = c.descendants().find(|n| n.has_tag_name("rootfile")).and_then(|n| n.attribute("full-path")).unwrap().to_owned();
    let opf_text = text(&opf_path);
    let opf = parse(&opf_text).unwrap_or_else(|e| panic!("{name}: package: {e}"));
    // Required metadata.
    for el in ["identifier", "title", "language"] {
        assert!(opf.descendants().any(|n| n.tag_name().name() == el && !n.text().unwrap_or("").trim().is_empty()), "{name}: dc:{el}");
    }
    assert!(opf.descendants().any(|n| n.attribute("property") == Some("dcterms:modified")), "{name}: modified");
    // The manifest: every file there, every XML file well-formed.
    let mut manifest = BTreeMap::new();
    let mut nav = None;
    for item in opf.descendants().filter(|n| n.has_tag_name("item")) {
        let href = resolve(&opf_path, item.attribute("href").unwrap());
        assert!(map.contains_key(&href), "{name}: {href} is in the manifest but not the zip");
        let media = item.attribute("media-type").unwrap();
        if media.contains("xml") {
            parse(&text(&href)).unwrap_or_else(|e| panic!("{name}: {href}: {e}"));
        }
        if item.attribute("properties").is_some_and(|p| p.split(' ').any(|x| x == "nav")) {
            nav = Some(href.clone());
        }
        manifest.insert(item.attribute("id").unwrap().to_owned(), (href, media.to_owned()));
    }
    // Every file in the zip is in the manifest (but the container's own).
    for f in map.keys() {
        assert!(
            f == "mimetype" || f.starts_with("META-INF/") || *f == opf_path || manifest.values().any(|(h, _)| h == f),
            "{name}: {f} is not in the manifest"
        );
    }
    // The spine: XHTML documents of the manifest, each once.
    let spine: Vec<&str> = opf.descendants().filter(|n| n.has_tag_name("itemref")).map(|n| n.attribute("idref").unwrap()).collect();
    assert!(!spine.is_empty(), "{name}");
    let mut seen = std::collections::BTreeSet::new();
    for id in &spine {
        let (_, media) = manifest.get(*id).unwrap_or_else(|| panic!("{name}: spine names {id}"));
        assert_eq!(media, "application/xhtml+xml", "{name}: {id}");
        assert!(seen.insert(*id), "{name}: {id} twice in the spine");
    }
    // Every navigation target, in the navigation document and the NCX, exists.
    let nav = nav.unwrap_or_else(|| panic!("{name}: no navigation document"));
    let mut targets = 0;
    let mut check_target = |from: &str, href: &str| {
        let (file, frag) = href.split_once('#').unwrap_or((href, ""));
        let path = resolve(from, file);
        let body = text(&path);
        if !frag.is_empty() {
            let doc = parse(&body).unwrap();
            assert!(ids(&doc).iter().any(|i| i == frag), "{name}: {href} from {from}: no such id");
        }
        targets += 1;
    };
    let nav_text = text(&nav);
    let nav_doc = parse(&nav_text).unwrap();
    let ops = "http://www.idpf.org/2007/ops";
    let tocs: Vec<_> = nav_doc.descendants().filter(|n| n.has_tag_name("nav") && n.attribute((ops, "type")) == Some("toc")).collect();
    assert_eq!(tocs.len(), 1, "{name}: one toc nav");
    let toc_links: Vec<String> = tocs[0].descendants().filter(|n| n.has_tag_name("a")).map(|n| n.attribute("href").unwrap().to_owned()).collect();
    assert!(!toc_links.is_empty(), "{name}");
    for a in nav_doc.descendants().filter(|n| n.has_tag_name("a")) {
        check_target(&nav, a.attribute("href").unwrap());
    }
    let landmarks: Vec<&str> = nav_doc
        .descendants()
        .filter(|n| n.has_tag_name("nav") && n.attribute((ops, "type")) == Some("landmarks"))
        .flat_map(|n| n.descendants().filter(|a| a.has_tag_name("a")).filter_map(|a| a.attribute((ops, "type"))))
        .collect();
    assert!(landmarks.contains(&"bodymatter"), "{name}: {landmarks:?}");
    let (ncx_path, _) = manifest.values().find(|(_, m)| m == "application/x-dtbncx+xml").unwrap_or_else(|| panic!("{name}: no NCX")).clone();
    let ncx_text = text(&ncx_path);
    let ncx = parse(&ncx_text).unwrap();
    let ncx_links: Vec<String> = ncx.descendants().filter(|n| n.has_tag_name("content")).map(|n| n.attribute("src").unwrap().to_owned()).collect();
    assert_eq!(ncx_links, toc_links, "{name}: the NCX has the toc's entries");
    for l in &ncx_links {
        check_target(&ncx_path, l);
    }
    // Every link in every document goes somewhere that exists.
    for (path, media) in manifest.values() {
        if media != "application/xhtml+xml" || *path == nav {
            continue;
        }
        let body = text(path);
        let doc = parse(&body).unwrap();
        let own = ids(&doc);
        for a in doc.descendants().filter(|n| n.has_tag_name("a")) {
            let href = a.attribute("href").unwrap();
            match href.strip_prefix('#') {
                Some(frag) => assert!(own.iter().any(|i| i == frag), "{name}: {path}: no {href}"),
                None => check_target(path, href),
            }
        }
        let lang = doc.root_element().attribute("lang");
        assert!(lang.is_some_and(|l| !l.is_empty()), "{name}: {path}: no language");
        assert_eq!(doc.root_element().attribute(("http://www.w3.org/XML/1998/namespace", "lang")), lang, "{name}: {path}");
    }
    (spine.len(), targets)
}

fn ebook_of(b: &Book, design: &str) -> Vec<u8> {
    let mut p = project();
    p.design.id = design.into();
    build::ebook(&p, &Default::default(), b, &Edition::ebook(), &|_| None).unwrap_or_else(|e| panic!("{design}: {e}")).1.bytes
}

fn imported(name: &str, format: Format) -> Book {
    let bytes = std::fs::read(novels().join(name)).unwrap();
    let stem = name.split('.').next().unwrap();
    let i = wordsmith_book_import::import(format, &bytes, &Default::default(), stem).unwrap_or_else(|e| panic!("{name}: {e}"));
    book::from_project(&i.project)
}

#[test]
fn every_fixture_gives_a_well_formed_package_whose_navigation_targets_exist() {
    let mut books: Vec<(String, Book)> = vec![("the Manuscript fixture".into(), gulliver())];
    for (name, format) in [
        ("waverley.docx", Format::Docx),
        ("waverley.odt", Format::Odt),
        ("waverley.rtf", Format::Rtf),
        ("waverley.md", Format::Markdown),
        ("waverley.txt", Format::Text),
        ("gullivers-travels.docx", Format::Docx),
        ("gullivers-travels.odt", Format::Odt),
        ("gullivers-travels.rtf", Format::Rtf),
        ("gullivers-travels.md", Format::Markdown),
        ("gullivers-travels.txt", Format::Text),
        ("gullivers-travels-online.docx", Format::Docx),
        ("middlemarch.docx", Format::Docx),
        ("middlemarch.txt", Format::Text),
    ] {
        books.push((name.into(), imported(name, format)));
    }
    for (name, b) in &books {
        let (docs, targets) = check_structure(name, &ebook_of(b, "classic"));
        let chapters = b.body.len();
        assert!(docs > chapters / 2 && targets > chapters / 2, "{name}: {docs} documents, {targets} targets for {chapters} sections");
    }
}

#[test]
fn every_design_gives_a_package_and_the_same_bytes_twice() {
    let b = gulliver();
    for d in builtin::all() {
        let a = ebook_of(&b, &d.id);
        check_structure(&d.id, &a);
        assert_eq!(a, ebook_of(&b, &d.id), "{}: two builds differ", d.id);
    }
}

#[test]
fn the_package_names_the_book_and_nests_chapters_under_parts() {
    let b = gulliver();
    let mut p = project();
    p.design.id = "classic".into();
    let (a, e) = build::ebook(&p, &Default::default(), &b, &Edition::ebook(), &|_| None).unwrap();
    let pk = &e.package;
    assert_eq!(pk.metadata.title, a.meta.title);
    assert!(pk.metadata.identifier.starts_with("urn:uuid:"), "no ISBN, so a UUID");
    let mut with_isbn = Edition::ebook();
    with_isbn.isbn = "978-0-306-40615-7".into();
    let (_, e2) = build::ebook(&p, &Default::default(), &b, &with_isbn, &|_| None).unwrap();
    assert_eq!(e2.package.metadata.identifier, "urn:isbn:9780306406157");
    // Gulliver's Travels has four parts, each holding its chapters.
    let parts: Vec<_> = pk.toc.iter().filter(|n| !n.children.is_empty()).collect();
    assert_eq!(parts.len(), 4, "{:?}", pk.toc.iter().map(|n| &n.label).collect::<Vec<_>>());
    assert!(parts.iter().all(|p| p.children.len() >= 7));
    // The half title is a printed book's page; the contents are the navigation document.
    assert!(pk.nav_in_spine.is_some());
    let files = unzip(&e.bytes);
    let all: String = files.iter().filter(|f| f.0.ends_with(".xhtml")).map(|f| String::from_utf8_lossy(&f.2).into_owned()).collect();
    assert!(!all.contains("halftitlepage"));
    assert!(all.contains("epub:type=\"chapter\" role=\"doc-chapter\""));
    assert!(all.contains("epub:type=\"noteref\""), "the fixture's notes are linked");
    // A print edition is not an ebook.
    assert!(build::ebook(&p, &Default::default(), &b, &Edition::paperback(), &|_| None).is_err());
}

#[test]
#[ignore = "writes the fixture's EPUB to the target folder for a person to look at"]
fn epub_for_review() {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/review");
    std::fs::create_dir_all(&out).unwrap();
    for d in builtin::all() {
        std::fs::write(out.join(format!("gulliver-{}.epub", d.id)), ebook_of(&gulliver(), &d.id)).unwrap();
    }
}
