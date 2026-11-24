// SPDX-License-Identifier: MIT OR Apache-2.0
//! Number styles (plan M3.2) for every number from 1 to 120.

use std::collections::BTreeSet;

use wordsmith_book_model::NumberStyle;
use wordsmith_book_model::numbers::{label, roman, words};

fn from_roman(s: &str) -> u32 {
    let v = |c| match c {
        'I' => 1,
        'V' => 5,
        'X' => 10,
        'L' => 50,
        'C' => 100,
        'D' => 500,
        'M' => 1000,
        _ => panic!("not roman: {c}"),
    };
    let chars: Vec<u32> = s.chars().map(v).collect();
    let mut total = 0;
    for (i, x) in chars.iter().enumerate() {
        if chars.get(i + 1).is_some_and(|n| n > x) { total -= *x as i64 } else { total += *x as i64 }
    }
    u32::try_from(total).unwrap()
}

fn from_words(s: &str) -> u32 {
    let ones = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    let tens = ["", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety"];
    let mut total = 0;
    for w in s.to_lowercase().split([' ', '-']) {
        if w == "hundred" {
            total *= 100;
        } else if let Some(n) = ones.iter().position(|o| *o == w) {
            total += n as u32;
        } else if let Some(n) = tens.iter().position(|t| *t == w) {
            total += 10 * n as u32;
        } else {
            panic!("not a number word: {w}");
        }
    }
    total
}

#[test]
fn every_style_for_1_to_120() {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for n in 1..=120 {
        let w = words(n);
        assert_eq!(from_words(&w), n, "{w}");
        assert!(w.split([' ', '-']).all(|x| x.starts_with(|c: char| c.is_uppercase())), "{w}");
        assert!(seen.insert(w.clone()));
        let r = roman(n);
        assert_eq!(from_roman(&r), n, "{r}");
        assert!(!r.contains("IIII") && !r.contains("VV") && !r.contains("XXXX"), "{r}");
        assert_eq!(label(NumberStyle::LabelWords, n, "Chapter"), Some(format!("Chapter {w}")));
        assert_eq!(label(NumberStyle::LabelFigures, n, "Chapter"), Some(format!("Chapter {n}")));
        assert_eq!(label(NumberStyle::Words, n, "Chapter"), Some(w.clone()));
        assert_eq!(label(NumberStyle::Figures, n, "Chapter"), Some(n.to_string()));
        assert_eq!(label(NumberStyle::Roman, n, "Chapter"), Some(r));
        assert_eq!(label(NumberStyle::None, n, "Chapter"), None);
    }
    assert_eq!(words(99), "Ninety-Nine");
    assert_eq!(words(115), "One Hundred Fifteen");
}
