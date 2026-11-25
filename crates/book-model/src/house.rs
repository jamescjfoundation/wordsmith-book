// SPDX-License-Identifier: MIT OR Apache-2.0
//! A publishing house's shared details (DSN-7, META-3 to META-5): its imprints, its series and
//! the ISBNs it owns, in one `house.toml` the user keeps where they like (its path in
//! Preferences). A book joins an imprint and a series by id and keeps a copy of what it takes
//! from them (the imprint's details, the house design and its lock), so it builds the same
//! without the house file.

use serde::{Deserialize, Serialize};

use crate::DesignChoice;

/// The house file's version.
pub const HOUSE_SCHEMA: u32 = 1;
/// Most imprints, series and ISBNs a house file holds (each).
pub const MAX_HOUSE_ITEMS: usize = 10_000;

/// A publishing house's shared details.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct House {
    /// The file's version ([`HOUSE_SCHEMA`]).
    pub schema: u32,
    /// The house's name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Its imprints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imprints: Vec<HouseImprint>,
    /// Its series.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<Series>,
    /// The ISBNs it owns, and which edition uses each.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub isbns: Vec<HouseIsbn>,
}

/// One imprint: its details for the title and copyright pages and its house design.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HouseImprint {
    /// Its id (lowercase, hyphens).
    pub id: String,
    /// Its name.
    pub name: String,
    /// Address lines.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub address: Vec<String>,
    /// Its website.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub website: String,
    /// Its logo: a PNG or JPEG, by its path from the house file's folder.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub logo: String,
    /// The copyright page's wording, with the usual placeholders (`{year}`, `{holder}`,
    /// `{imprint}`, ...); empty: Book's.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub copyright: String,
    /// Its house design: a built-in design's id, or a design file's id in the house folder's
    /// `designs/`; empty: none.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub design: String,
    /// Its books must keep the house design (DSN-7).
    #[serde(skip_serializing_if = "is_false")]
    pub lock_design: bool,
}

/// One series: its books in order, one design across them.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Series {
    /// Its id.
    pub id: String,
    /// Its name.
    pub name: String,
    /// The imprint it is published under, by id (empty: none).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub imprint: String,
    /// Its design (as an imprint's), overriding the imprint's.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub design: String,
    /// Its books must keep the series design (DSN-7).
    #[serde(skip_serializing_if = "is_false")]
    pub lock_design: bool,
    /// Its books, in order.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub books: Vec<SeriesBook>,
}

/// One book of a series.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeriesBook {
    /// Its number in the series (1 for the first).
    pub number: u32,
    /// Its title.
    pub title: String,
    /// Its Book project, by its path from the house file's folder (empty: not made in Book).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub project: String,
}

/// One ISBN the house owns.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HouseIsbn {
    /// The ISBN-13, as written.
    pub isbn: String,
    /// The book that uses it, by title (empty: free).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub book: String,
    /// The edition that uses it, by name.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub edition: String,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// What a book takes from its house: which imprint and series it is in, and the design lock.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HouseLink {
    /// Its imprint, by id (empty: none).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub imprint: String,
    /// Its series, by id (empty: none).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub series: String,
    /// Its number in the series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_number: Option<u32>,
    /// The house design it must keep, when its imprint or series locks one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock: Option<DesignLock>,
}

/// A design a book must keep (DSN-7).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DesignLock {
    /// Who locks it: "the imprint “…”" or "the series “…”".
    pub by: String,
    /// The design.
    pub design: DesignChoice,
}

fn id_ok(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

impl House {
    /// What is wrong with the house file, one line each.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.schema != HOUSE_SCHEMA {
            out.push(format!("The house file is version {}; Book reads version {HOUSE_SCHEMA}.", self.schema));
        }
        if self.imprints.len() > MAX_HOUSE_ITEMS || self.series.len() > MAX_HOUSE_ITEMS || self.isbns.len() > MAX_HOUSE_ITEMS {
            out.push(format!("The house file has more than {MAX_HOUSE_ITEMS} imprints, series or ISBNs."));
        }
        let mut seen = std::collections::BTreeSet::new();
        for id in self.imprints.iter().map(|i| &i.id).chain(self.series.iter().map(|s| &s.id)) {
            if !id_ok(id) {
                out.push(format!("“{id}” is not an id: lowercase letters, digits and hyphens."));
            } else if !seen.insert(id.clone()) {
                out.push(format!("Two imprints or series have the id “{id}”."));
            }
        }
        // ISBNs (META-5): each valid and owned once; an edition uses one.
        let mut owned = std::collections::BTreeSet::new();
        let mut uses = std::collections::BTreeSet::new();
        for i in &self.isbns {
            match crate::isbn::check(&i.isbn) {
                Err(e) => out.push(format!("The house's ISBN list: {e}")),
                Ok(d) => {
                    if !owned.insert(d) {
                        out.push(format!("The house's ISBN list has {} twice.", i.isbn.trim()));
                    }
                }
            }
            if !i.book.trim().is_empty() && !uses.insert((i.book.trim().to_lowercase(), i.edition.trim().to_lowercase())) {
                out.push(format!("The house's ISBN list gives “{}” ({}) two ISBNs.", i.book.trim(), i.edition.trim()));
            }
        }
        for s in &self.series {
            if !s.imprint.is_empty() && !self.imprints.iter().any(|i| i.id == s.imprint) {
                out.push(format!("The series “{}” names an imprint the house does not have: “{}”.", s.name, s.imprint));
            }
            let mut numbers = std::collections::BTreeSet::new();
            for b in &s.books {
                if !numbers.insert(b.number) {
                    out.push(format!("The series “{}” has two books numbered {}.", s.name, b.number));
                }
            }
        }
        out
    }

    /// An imprint by id.
    pub fn imprint(&self, id: &str) -> Option<&HouseImprint> {
        self.imprints.iter().find(|i| i.id == id)
    }

    /// A series by id.
    pub fn series(&self, id: &str) -> Option<&Series> {
        self.series.iter().find(|s| s.id == id)
    }
}
