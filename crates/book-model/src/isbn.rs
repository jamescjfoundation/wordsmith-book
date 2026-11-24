// SPDX-License-Identifier: MIT OR Apache-2.0
//! ISBNs (ISO 2108, <https://www.iso.org/standard/65483.html>): ISBN-13 with its mod-10 check
//! digit (weights 1 and 3), ISBN-10 with its mod-11 check character (weights 10 to 2, `X` for
//! 10). Hyphens and spaces are ignored when checking.

/// The digits of an ISBN as written, without hyphens or spaces (an `X` kept).
pub fn digits(s: &str) -> String {
    s.chars().filter(|c| !matches!(c, '-' | ' ' | '\u{2010}' | '\u{2011}')).map(|c| c.to_ascii_uppercase()).collect()
}

/// The check digit an ISBN-13 with these first twelve digits needs.
pub fn check13(first12: &[u32]) -> u32 {
    let sum: u32 = first12.iter().enumerate().map(|(i, d)| if i % 2 == 0 { *d } else { 3 * d }).sum();
    (10 - sum % 10) % 10
}

/// The check character an ISBN-10 with these first nine digits needs (10 is `X`).
pub fn check10(first9: &[u32]) -> u32 {
    let sum: u32 = first9.iter().enumerate().map(|(i, d)| (10 - i as u32) * d).sum();
    (11 - sum % 11) % 11
}

/// Check an ISBN; gives its thirteen digits, or why it is not valid in plain words.
pub fn check(s: &str) -> Result<String, String> {
    let d = digits(s);
    if d.is_empty() {
        return Err("No ISBN is given.".into());
    }
    if d.len() == 13 {
        if !d.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("“{s}” has characters that are not digits."));
        }
        if !(d.starts_with("978") || d.starts_with("979")) {
            return Err(format!("“{s}” does not start with 978 or 979, as every ISBN-13 does."));
        }
        let n: Vec<u32> = d.chars().filter_map(|c| c.to_digit(10)).collect();
        if check13(&n[..12]) != n[12] {
            return Err(format!("“{s}” has the wrong check digit (the last digit should be {}). A digit was probably mistyped.", check13(&n[..12])));
        }
        return Ok(d);
    }
    if d.len() == 10 {
        let (body, last) = d.split_at(9);
        if !body.chars().all(|c| c.is_ascii_digit()) || !(last == "X" || last.chars().all(|c| c.is_ascii_digit())) {
            return Err(format!("“{s}” is not an ISBN-10: nine digits and a digit or X."));
        }
        let n: Vec<u32> = body.chars().filter_map(|c| c.to_digit(10)).collect();
        let want = check10(&n);
        let got = if last == "X" { 10 } else { last.chars().next().and_then(|c| c.to_digit(10)).unwrap_or(99) };
        if want != got {
            return Err(format!("“{s}” has the wrong check character (it should be {}).", if want == 10 { "X".to_owned() } else { want.to_string() }));
        }
        return Ok(to13(&n));
    }
    Err(format!("“{s}” has {} digits; an ISBN has 13 (or 10, for an older one).", d.len()))
}

fn to13(first9: &[u32]) -> String {
    let mut n = vec![9, 7, 8];
    n.extend_from_slice(first9);
    let c = check13(&n);
    n.push(c);
    n.iter().map(u32::to_string).collect()
}

/// An ISBN-13's digits hyphenated in the common 3-1-3-5-1 grouping for display when the
/// registration group is not known (the exact grouping depends on ranges the national agencies
/// publish).
pub fn display(d13: &str) -> String {
    if d13.len() != 13 {
        return d13.to_owned();
    }
    format!("{}-{}-{}-{}-{}", &d13[..3], &d13[3..4], &d13[4..7], &d13[7..12], &d13[12..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isbn_13_and_10() {
        assert_eq!(check("978-0-306-40615-7"), Ok("9780306406157".into()));
        assert!(check("978-0-306-40615-8").unwrap_err().contains("check digit"));
        assert_eq!(check("0-306-40615-2"), Ok("9780306406157".into()));
        assert!(check("0-306-40615-3").is_err());
        assert_eq!(check("0-8044-2957-X"), Ok("9780804429573".into()));
        assert!(check("123").is_err());
        assert!(check("").is_err());
        assert!(check("1230306406157").unwrap_err().contains("978"));
    }
}
