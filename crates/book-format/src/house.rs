// SPDX-License-Identifier: MIT OR Apache-2.0
//! The house file (`house.toml`, DSN-7 and META-3 to META-5): read with the limits every
//! project file has, checked, and written whole or not at all. Its pictures and designs are
//! files beside it: logos by path, designs in `designs/<id>.toml`.

use std::path::{Path, PathBuf};

use wordsmith_book_model::house::{HOUSE_SCHEMA, House};

use crate::Error;

/// The largest house file read, in bytes.
pub const MAX_HOUSE: u64 = 4 << 20;
/// The largest file read beside it (a logo or a design), in bytes.
pub const MAX_HOUSE_FILE: u64 = 16 << 20;

/// Read a house file; a damaged one or one with problems is refused.
pub fn read(path: &Path) -> Result<House, Error> {
    let bytes =
        jcj_project::files::read_bounded(path, MAX_HOUSE).map_err(|e| Error::Damaged { file: path.display().to_string(), why: e.to_string() })?;
    let text = String::from_utf8(bytes).map_err(|_| Error::Damaged { file: path.display().to_string(), why: "it is not UTF-8".into() })?;
    let table = crate::toml_file::parse_table(&text, "house.toml", MAX_HOUSE)?;
    let found = table.get("schema").and_then(toml::Value::as_integer).unwrap_or(0);
    if found > i64::from(HOUSE_SCHEMA) {
        return Err(Error::Newer { found: u32::try_from(found).unwrap_or(u32::MAX), supported: HOUSE_SCHEMA });
    }
    let house: House = table.try_into().map_err(|e: toml::de::Error| Error::Damaged { file: "house.toml".into(), why: e.message().to_owned() })?;
    if let Some(p) = house.problems().into_iter().next() {
        return Err(Error::Damaged { file: "house.toml".into(), why: p });
    }
    Ok(house)
}

/// Write a house file (checked first).
pub fn write(path: &Path, house: &House) -> Result<(), Error> {
    if let Some(p) = house.problems().into_iter().next() {
        return Err(Error::Damaged { file: "house.toml".into(), why: p });
    }
    let text = toml::to_string(house).map_err(|e| Error::Damaged { file: "house.toml".into(), why: e.to_string() })?;
    jcj_project::files::write_atomic(path, text.as_bytes()).map_err(|e| Error::Damaged { file: path.display().to_string(), why: e.to_string() })
}

/// A file beside the house file, by its path from the house file's folder (no `..`, not
/// absolute), bounded.
pub fn beside(house_file: &Path, rel: &str) -> Result<Vec<u8>, Error> {
    let rel = rel.trim();
    if rel.is_empty() || rel.contains("..") || rel.starts_with('/') || rel.starts_with('\\') || rel.contains(':') {
        return Err(Error::Damaged { file: rel.into(), why: "a house file's pictures and designs are beside it".into() });
    }
    let path: PathBuf = house_file.parent().map_or_else(|| PathBuf::from(rel), |d| d.join(rel));
    jcj_project::files::read_bounded(&path, MAX_HOUSE_FILE).map_err(|e| Error::Damaged { file: rel.into(), why: e.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordsmith_book_model::house::{HouseImprint, Series, SeriesBook};

    #[test]
    fn a_house_round_trips_and_bad_ones_are_refused() {
        let dir = std::env::temp_dir().join(format!("wordsmith-book-house-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("house.toml");
        let house = House {
            schema: HOUSE_SCHEMA,
            name: "Lantern House".into(),
            imprints: vec![HouseImprint {
                id: "lantern".into(),
                name: "Lantern Press".into(),
                design: "classic".into(),
                lock_design: true,
                ..HouseImprint::default()
            }],
            series: vec![Series {
                id: "voyages".into(),
                name: "Voyages".into(),
                imprint: "lantern".into(),
                books: vec![SeriesBook { number: 1, title: "One".into(), project: String::new() }],
                ..Series::default()
            }],
            isbns: Vec::new(),
        };
        write(&path, &house).unwrap();
        assert_eq!(read(&path).unwrap(), house);
        std::fs::write(&path, "schema = 1\n[[imprints]]\nid = \"Bad Id\"\nname = \"x\"\n").unwrap();
        assert!(read(&path).is_err());
        std::fs::write(&path, "schema = 9\n").unwrap();
        assert!(matches!(read(&path), Err(Error::Newer { .. })));
        assert!(beside(&path, "../secret").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
