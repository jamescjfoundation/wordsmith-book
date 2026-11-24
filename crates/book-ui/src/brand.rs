// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wordsmith Book's look: its design tokens (`assets/brand/book/tokens.json`), its mark and its
//! app icon. The brand lives here, never in the core. The mark is the same open book that
//! `tools/brand-icons.py` draws for the icons.

use jcj_shell::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2};
use jcj_shell::theme::Tokens;

/// The tokens file, as shipped.
pub const TOKENS_JSON: &str = include_str!("../../../assets/brand/book/tokens.json");

/// The app icon at 256 pixels (PNG), for the window.
pub const ICON_256: &[u8] = include_bytes!("../../../assets/brand/book/icon-256.png");

/// The ribbon marker's color (the palette's gilt).
pub const GILT: Color32 = Color32::from_rgb(201, 162, 74);

/// Book's tokens; the neutral set if the shipped file could not be read (a test guards it).
pub fn tokens() -> Tokens {
    Tokens::from_json(TOKENS_JSON).unwrap_or_default()
}

fn page(side: f32) -> Vec<(f32, f32)> {
    let steps = 12;
    let mut pts = Vec::with_capacity(2 * steps + 2);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        pts.push((50.0 + side * 40.0 * t, 30.0 - 8.0 * (std::f32::consts::PI * t).sin() + 2.0 * t));
    }
    for i in 0..=steps {
        let t = 1.0 - i as f32 / steps as f32;
        pts.push((50.0 + side * 40.0 * t, 76.0 - 6.0 * (std::f32::consts::PI * t).sin() + 2.0 * t));
    }
    pts
}

/// The mark's width at `height` points (it is drawn in a square).
pub fn mark_width(height: f32) -> f32 {
    height
}

/// Paint the mark (an open book with a ribbon) in `rect`, outlined in `ink`.
pub fn paint_mark(painter: &Painter, rect: Rect, ink: Color32) {
    let s = rect.width().min(rect.height()) / 100.0;
    let at = |(x, y): (f32, f32)| -> Pos2 { pos2(rect.min.x + x * s, rect.min.y + y * s) };
    let stroke = Stroke::new((3.0 * s).max(1.0), ink);
    for side in [-1.0, 1.0] {
        let pts: Vec<Pos2> = page(side).into_iter().map(at).collect();
        painter.add(Shape::closed_line(pts, stroke));
    }
    painter.line_segment([at((50.0, 28.0)), at((50.0, 78.0))], stroke);
    let ribbon = [(53.0, 30.0), (59.0, 30.0), (59.0, 86.0), (56.0, 82.0), (53.0, 86.0)];
    painter.add(Shape::convex_polygon(ribbon.into_iter().map(at).collect(), GILT, Stroke::NONE));
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcj_shell::theme::Mode;

    #[test]
    fn the_shipped_tokens_read_and_differ_from_the_neutral_set() {
        let t = Tokens::from_json(TOKENS_JSON).expect("tokens.json reads");
        let neutral = Tokens::default();
        for mode in [Mode::Light, Mode::Dark, Mode::ExtraDark] {
            assert_ne!(t.colors(mode), neutral.colors(mode), "{mode:?} has Book's own colors");
        }
    }

    #[test]
    fn the_icon_is_a_png() {
        assert_eq!(&ICON_256[..8], b"\x89PNG\r\n\x1a\n");
    }
}
