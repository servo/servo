/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use font::{FontHandleMethods, FontMetrics, FontTableMethods};
use font::{FontTableTag, FractionalPixel};
use platform::font_context::FontContextHandle;
use platform::font_template::FontTemplateData;
use std::sync::Arc;
use style::computed_values::{font_stretch, font_weight};
use text::glyph::GlyphId;

#[derive(Debug)]
pub struct FontTable {
    buffer: Vec<u8>,
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

#[derive(Debug)]
pub struct FontHandle {
    handle: FontContextHandle,
}

impl Drop for FontHandle {
    fn drop(&mut self) {
    }
}

impl FontHandleMethods for FontHandle {
    fn new_from_template(fctx: &FontContextHandle,
                         template: Arc<FontTemplateData>,
                         pt_size: Option<Au>)
                         -> Result<FontHandle, ()> {
        Err(())
    }

    fn template(&self) -> Arc<FontTemplateData> {
        unimplemented!()
    }
    fn family_name(&self) -> String {
        String::from("Unknown")
    }
    fn face_name(&self) -> String {
        String::from("Unknown")
    }
    fn is_italic(&self) -> bool {
        false
    }
    fn boldness(&self) -> font_weight::T {
        font_weight::T::Weight400
    }
    fn stretchiness(&self) -> font_stretch::T {
        font_stretch::T::normal
    }
    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        None
    }
    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId)
                       -> FractionalPixel {
        0.0
    }
    fn can_do_fast_shaping(&self) -> bool {
        false
    }
    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        None
    }
    fn metrics(&self) -> FontMetrics {
        unimplemented!()
    }
    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        None
    }
}
