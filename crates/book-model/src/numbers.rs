// SPDX-License-Identifier: MIT OR Apache-2.0
//! Chapter and part numbers (STR-2, STR-3): in English words ("Twenty-One", "One Hundred
//! Twenty"), figures, or roman numerals, with or without a label ("Chapter").

use crate::NumberStyle;

const ONES: [&str; 20] = [
    "Zero",
    "One",
    "Two",
    "Three",
    "Four",
    "Five",
    "Six",
    "Seven",
    "Eight",
    "Nine",
    "Ten",
    "Eleven",
    "Twelve",
    "Thirteen",
    "Fourteen",
    "Fifteen",
    "Sixteen",
    "Seventeen",
    "Eighteen",
    "Nineteen",
];
const TENS: [&str; 10] = ["", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety"];

/// `n` in English words, each word capitalised, tens and units joined by a hyphen.
pub fn words(n: u32) -> String {
    match n {
        0..=19 => ONES[n as usize].to_owned(),
        20..=99 => {
            let t = TENS[(n / 10) as usize];
            if n.is_multiple_of(10) { t.to_owned() } else { format!("{t}-{}", ONES[(n % 10) as usize]) }
        }
        100..=999 => {
            let h = format!("{} Hundred", ONES[(n / 100) as usize]);
            if n.is_multiple_of(100) { h } else { format!("{h} {}", words(n % 100)) }
        }
        _ => {
            let th = format!("{} Thousand", words(n / 1000));
            if n.is_multiple_of(1000) { th } else { format!("{th} {}", words(n % 1000)) }
        }
    }
}

/// `n` in upper-case roman numerals (1 to 3,999; figures beyond).
pub fn roman(n: u32) -> String {
    if n == 0 || n >= 4000 {
        return n.to_string();
    }
    const TABLE: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut n = n;
    let mut out = String::new();
    for (v, s) in TABLE {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

/// The number line for `n` in `style`, with `label` ("Chapter") where the style has one; none for
/// [`NumberStyle::None`].
pub fn label(style: NumberStyle, n: u32, label: &str) -> Option<String> {
    Some(match style {
        NumberStyle::LabelWords => format!("{label} {}", words(n)),
        NumberStyle::LabelFigures => format!("{label} {n}"),
        NumberStyle::Words => words(n),
        NumberStyle::Figures => n.to_string(),
        NumberStyle::Roman => roman(n),
        NumberStyle::None => return None,
    })
}

/// Whether a title is only a number line ("Chapter 1", "CHAPTER XII.", "Part Two", "12"), so the
/// title itself numbers the section and no number line is added.
pub fn is_number_line(title: &str) -> bool {
    let t = title.trim().trim_end_matches(['.', ':']);
    let mut parts = t.split_whitespace();
    let (Some(first), second, None) = (parts.next(), parts.next(), parts.next()) else { return false };
    let num = |s: &str| {
        let s = s.trim_end_matches(['.', ':']);
        s.chars().all(|c| c.is_ascii_digit()) && !s.is_empty()
            || (!s.is_empty() && s.chars().all(|c| "IVXLCDMivxlcdm".contains(c)))
            || (1..=120).any(|n| words(n).eq_ignore_ascii_case(s))
    };
    match second {
        None => num(first),
        Some(n) => ["chapter", "part", "book", "volume"].contains(&first.to_lowercase().as_str()) && num(n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_and_roman() {
        assert_eq!(words(1), "One");
        assert_eq!(words(21), "Twenty-One");
        assert_eq!(words(100), "One Hundred");
        assert_eq!(words(120), "One Hundred Twenty");
        assert_eq!(roman(4), "IV");
        assert_eq!(roman(1994), "MCMXCIV");
        assert!(is_number_line("CHAPTER XII.") && is_number_line("Chapter 1") && is_number_line("Part Two") && is_number_line("12"));
        assert!(!is_number_line("Chapter One: The Sea") && !is_number_line("The Sea"));
    }
}
