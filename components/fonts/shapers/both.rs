/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::Write as _;

use skrifa::Tag;

use super::harfbuzz::HarfbuzzGlyphShapingResult;
use super::harfrust::HarfrustGlyphShapingResult;
use super::{GlyphShapingResult, HarfBuzzShaper, HarfRustShaper};
use crate::{Font, FontBaseline, ShapedText, ShapingOptions};

pub(crate) struct ShapedGlyphData {
    buzz: HarfbuzzGlyphShapingResult,
    rust: HarfrustGlyphShapingResult,
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
        font_features: &[(Tag, u32)],
    ) -> ShapedGlyphData {
        ShapedGlyphData {
            buzz: self.buzz.shaped_glyph_data(text, options, font_features),
            rust: self.rust.shaped_glyph_data(text, options, font_features),
        }
    }

    pub fn shape_text(
        &self,
        text: &str,
        options: &ShapingOptions,
        font_features: &[(Tag, u32)],
    ) -> ShapedText {
        let glyph_data = self.shaped_glyph_data(text, options, font_features);
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

        ShapedText::with_shaped_glyph_data(text, options, &glyph_data.rust)
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

fn shape_data_eq(a: &impl GlyphShapingResult, b: &impl GlyphShapingResult) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for (a, b) in a.iter().zip(b.iter()) {
        if a != b {
            return false;
        }
    }

    true
}

fn log_shape_data(data: &impl GlyphShapingResult) {
    let mut out = std::io::stdout().lock();
    writeln!(&mut out, "len: {}", data.len()).unwrap();
    writeln!(&mut out, "offsets:").unwrap();
    for glyph in data.iter() {
        write!(&mut out, "{} ", glyph.string_byte_offset).unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "entries:").unwrap();
    for glyph in data.iter() {
        write!(&mut out, "cp: {} ad: {} ", glyph.glyph_id, glyph.advance.0).unwrap();
        match glyph.offset {
            Some(offset) => write!(&mut out, "Some(x:{}, y:{})", offset.x.0, offset.y.0).unwrap(),
            None => write!(&mut out, "None").unwrap(),
        };
        writeln!(&mut out).unwrap();
    }
}
