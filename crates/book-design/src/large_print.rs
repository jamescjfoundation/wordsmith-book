// SPDX-License-Identifier: MIT OR Apache-2.0
//! The large-print variant of any design (PRT-11): the same design, its type set at 16 pt and up.
//!
//! The body goes to 16 pt (or stays larger) on open leading, ragged right and without hyphens;
//! every other size grows in the same proportion, within the format's limits; asides are set no
//! smaller than the body; drop caps become a plain first line and small-capital lead-ins stay.
//! A large-print edition uses it whatever design the book has, so one click (adding the edition)
//! gives a large-print book.

use crate::design::{Design, FirstLine, Hyphenation};

/// The smallest body size of large print, points.
pub const LARGE_PRINT_SIZE: f32 = 16.0;
/// The smallest running-head size of large print, points.
pub const LARGE_PRINT_HEADS: f32 = 12.0;

fn scaled(v: f32, k: f32, lo: f32, hi: f32) -> f32 {
    ((v * k * 2.0).round() / 2.0).clamp(lo, hi)
}

/// The large-print variant of `d`.
pub fn variant(d: &Design) -> Design {
    let mut out = d.clone();
    let k = if d.body.size < LARGE_PRINT_SIZE { LARGE_PRINT_SIZE / d.body.size } else { 1.0 };
    out.id = format!("{}-large-print", d.id);
    out.name = format!("{} (large print)", d.name);
    out.body.size = d.body.size.max(LARGE_PRINT_SIZE);
    out.body.leading = d.body.leading.max(1.4);
    out.body.justify = false;
    out.body.hyphenation = Hyphenation::Off;
    out.chapter.number_size = scaled(d.chapter.number_size, k, LARGE_PRINT_SIZE, 72.0);
    out.chapter.title_size = scaled(d.chapter.title_size, k, LARGE_PRINT_SIZE, 72.0);
    // A drop cap's sunk lines crowd large type; the first line stays plain or in small capitals.
    if out.chapter.first_line == FirstLine::DropCap {
        out.chapter.first_line = FirstLine::None;
    }
    out.part.number_size = scaled(d.part.number_size, k, LARGE_PRINT_SIZE, 72.0);
    out.part.title_size = scaled(d.part.title_size, k, LARGE_PRINT_SIZE, 72.0);
    out.heads.size = scaled(d.heads.size, k, LARGE_PRINT_HEADS, 14.0);
    for a in [&mut out.asides.default, &mut out.asides.letter, &mut out.asides.note, &mut out.asides.sign, &mut out.asides.message] {
        a.size = a.size.max(1.0);
    }
    out
}
