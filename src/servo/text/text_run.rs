use geom::point::Point2D;
use geom::size::Size2D;
use gfx::geometry::{au, px_to_au};
use libc::{c_void};
use font_library::FontLibrary;
use font::Font;
use glyph::Glyph;
use shaper::shape_text;

/// A single, unbroken line of text
struct TextRun {
    priv text: ~str,
    priv glyphs: ~[Glyph],
    priv size_: Size2D<au>,
    priv min_break_width_: au,
}

impl TextRun {
    /// The size of the entire TextRun
    pure fn size() -> Size2D<au> { self.size_ }
    pure fn min_break_width() -> au { self.min_break_width_ }

    /// Split a run of text in two
    // FIXME: Should be storing a reference to the Font inside
    // of the TextRun, but I'm hitting cycle collector bugs
    fn split(font: &Font, h_offset: au) -> (TextRun, TextRun) {
        assert h_offset >= self.min_break_width();
        assert h_offset <= self.size_.width;

        let mut curr_run = ~"";

        for iter_indivisible_slices(font, self.text) |slice| {
            let mut candidate = curr_run;

            if candidate.is_not_empty() {
                candidate += " "; // FIXME: just inserting spaces between words can't be right
            }

            candidate += slice;

            let glyphs = shape_text(font, candidate);
            let size = glyph_run_size(glyphs);
            if size.width <= h_offset {
                curr_run = candidate;
            } else {
                break;
            }
        }

        assert curr_run.is_not_empty();

        let first = move curr_run;
        let second = str::slice(self.text, first.len(), self.text.len());
        let second = second.trim_left();
        return (TextRun(font, first), TextRun(font, second));
    }
}

fn TextRun(font: &Font, +text: ~str) -> TextRun {
    let glyphs = shape_text(font, text);
    let size = glyph_run_size(glyphs);
    let min_break_width = calc_min_break_width(font, text);

    TextRun {
        text: text,
        glyphs: shape_text(font, text),
        size_: size,
        min_break_width_: min_break_width
    }
}

fn glyph_run_size(glyphs: &[Glyph]) -> Size2D<au> {
    let height = px_to_au(20);
    let pen_start_x = px_to_au(0);
    let pen_start_y = height;
    let pen_start = Point2D(pen_start_x, pen_start_y);
    let pen_end = glyphs.foldl(pen_start, |cur, glyph| {
        Point2D(cur.x.add(glyph.pos.offset.x).add(glyph.pos.advance.x),
                cur.y.add(glyph.pos.offset.y).add(glyph.pos.advance.y))
    });
    return Size2D(pen_end.x, pen_end.y);
}

/// Discovers the width of the largest indivisible substring
fn calc_min_break_width(font: &Font, text: &str) -> au {
    let mut max_piece_width = au(0);
    for iter_indivisible_slices(font, text) |slice| {
        let glyphs = shape_text(font, slice);
        let size = glyph_run_size(glyphs);
        if size.width > max_piece_width {
            max_piece_width = size.width
        }
    }
    return max_piece_width;
}

/// Iterates over all the indivisible substrings
fn iter_indivisible_slices(font: &Font, text: &r/str,
                           f: fn((&r/str)) -> bool) {

    let mut curr = text;
    loop {
        match str::find(curr, |c| !char::is_whitespace(c) ) {
          Some(idx) => {
            curr = str::view(curr, idx, curr.len());
          }
          None => {
            // Everything else is whitespace
            break
          }
        }

        match str::find(curr, |c| char::is_whitespace(c) ) {
          Some(idx) => {
            let piece = str::view(curr, 0, idx);
            if !f(piece) { break }
            curr = str::view(curr, idx, curr.len());
          }
          None => {
            assert curr.is_not_empty();
            if !f(curr) { break }
            // This is the end of the string
            break;
          }
        }
    }
}

#[test]
fn test_calc_min_break_width1() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"firecracker");
    let expected = px_to_au(84);
    assert expected == actual;
}

#[test]
fn test_calc_min_break_width2() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"firecracker yumyum");
    let expected = px_to_au(84);
    assert expected == actual;
}

#[test]
fn test_calc_min_break_width3() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"yumyum firecracker");
    let expected = px_to_au(84);
    assert expected == actual;
}

#[test]
fn test_calc_min_break_width4() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let actual = calc_min_break_width(font, ~"yumyum firecracker yumyum");
    let expected = px_to_au(84);
    assert expected == actual;
}

#[test]
fn test_iter_indivisible_slices() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "firecracker yumyum woopwoop") |slice| {
        slices += [slice];
    }
    assert slices == ~["firecracker", "yumyum", "woopwoop"];
}

#[test]
fn test_iter_indivisible_slices_trailing_whitespace() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "firecracker  ") |slice| {
        slices += [slice];
    }
    assert slices == ~["firecracker"];
}

#[test]
fn test_iter_indivisible_slices_leading_whitespace() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "  firecracker") |slice| {
        slices += [slice];
    }
    assert slices == ~["firecracker"];
}

#[test]
fn test_iter_indivisible_slices_empty() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let mut slices = ~[];
    for iter_indivisible_slices(font, "") |slice| {
        slices += [slice];
    }
    assert slices == ~[];
}

#[test]
fn test_split() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker yumyum");
    let break_runs = run.split(font, run.min_break_width());
    assert break_runs.first().text == ~"firecracker";
    assert break_runs.second().text == ~"yumyum";
}

#[test]
fn test_split2() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker yum yum yum yum yum");
    let break_runs = run.split(font, run.min_break_width());
    assert break_runs.first().text == ~"firecracker";
    assert break_runs.second().text == ~"yum yum yum yum yum";
}

#[test]
fn test_split3() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker firecracker");
    let break_runs = run.split(font, run.min_break_width() + px_to_au(10));
    assert break_runs.first().text == ~"firecracker";
    assert break_runs.second().text == ~"firecracker";

}

#[test]
#[ignore(cfg(target_os = "macos"))]
fn should_calculate_the_total_size() {
    let flib = FontLibrary();
    let font = flib.get_test_font();
    let run = TextRun(font, ~"firecracker");
    let expected = Size2D(px_to_au(84), px_to_au(20));
    assert run.size() == expected;
}

