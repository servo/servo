use std::io::Write as _;

use app_units::Au;

use super::harfbuzz::ShapedGlyphData as HarfBuzzShapedGlyphData;
use super::harfrust::ShapedGlyphData as HarfRustShapedGlyphData;
use super::{HarfBuzzShapedGlyphData as THarfBuzzShapedGlyphData, HarfBuzzShaper, HarfRustShaper};
use crate::{Font, FontBaseline, GlyphStore, ShapingOptions};

pub(crate) struct ShapedGlyphData {
    buzz: HarfBuzzShapedGlyphData,
    rust: HarfRustShapedGlyphData,
}

pub(crate) struct Shaper {
    buzz: HarfBuzzShaper,
    rust: HarfRustShaper,
}

impl Shaper {
    pub(crate) fn new(font: &Font) -> Self {
        Self {
            buzz: HarfBuzzShaper::new(font),
            rust: HarfRustShaper::new(font),
        }
    }

    pub(crate) fn shaped_glyph_data(
        &self,
        text: &str,
        options: &ShapingOptions,
    ) -> ShapedGlyphData {
        ShapedGlyphData {
            buzz: self.buzz.shaped_glyph_data(text, options),
            rust: self.rust.shaped_glyph_data(text, options),
        }
    }

    pub fn shape_text(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        let glyph_data = self.shaped_glyph_data(text, options);
        let font = self.buzz.font();

        let equal = shape_data_eq(&glyph_data.buzz, &glyph_data.rust);
        if !equal {
            println!("SHAPING DATA DIFFERENT:");
            println!("Input text:");
            println!("{text}");
            println!("Buzz:");
            log_shape_data(&glyph_data.buzz);
            println!("Rust:");
            log_shape_data(&glyph_data.rust);
            println!("========================");
        }

        super::shape_text_harfbuzz(&glyph_data.rust, font, text, options, glyphs);
    }

    pub fn baseline(&self) -> Option<FontBaseline> {
        let buzz_baseline = self.buzz.baseline();
        let rust_baseline = self.rust.baseline();

        // Debug log
        let equal = buzz_baseline == rust_baseline;
        let eq_word = if equal { "same" } else { "diff" };
        println!(
            "BL ({eq_word}) C: {:?} | R: {:?}",
            buzz_baseline, rust_baseline
        );

        rust_baseline
    }
}

fn shape_data_eq(a: &impl THarfBuzzShapedGlyphData, b: &impl THarfBuzzShapedGlyphData) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut a_y_pos = Au::new(0);
    let mut b_y_pos = Au::new(0);
    for i in 0..a.len() {
        if a.byte_offset_of_glyph(i) != b.byte_offset_of_glyph(i) {
            return false;
        }

        if a.entry_for_glyph(i, &mut a_y_pos) != b.entry_for_glyph(i, &mut b_y_pos) {
            return false;
        }
    }

    true
}

fn log_shape_data(data: &impl THarfBuzzShapedGlyphData) {
    let mut out = std::io::stdout().lock();
    writeln!(&mut out, "len: {}", data.len()).unwrap();
    writeln!(&mut out, "offsets:").unwrap();
    for i in 0..data.len() {
        write!(&mut out, "{} ", data.byte_offset_of_glyph(i)).unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "entries:").unwrap();
    let mut y_pos = Au::new(0);
    for i in 0..data.len() {
        let entry = data.entry_for_glyph(i, &mut y_pos);
        write!(&mut out, "cp: {} ad: {} ", entry.codepoint, entry.advance.0).unwrap();
        match entry.offset {
            Some(offset) => write!(&mut out, "Some(x:{}, y:{})", offset.x.0, offset.y.0).unwrap(),
            None => write!(&mut out, "None").unwrap(),
        };
        writeln!(&mut out).unwrap();
    }
}
