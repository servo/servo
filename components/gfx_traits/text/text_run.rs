/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use font::FontMetrics;
use platform::font_template::FontTemplateData;
use std::sync::Arc;
use text::glyph::{CharIndex, GlyphStore};
use util::geometry::Au;

/// A single "paragraph" of text in one font size and style.
#[derive(Clone, Deserialize, Serialize)]
pub struct TextRun {
    /// The UTF-8 string represented by this text run.
    pub text: Arc<String>,
    pub font_template: Arc<FontTemplateData>,
    pub actual_pt_size: Au,
    pub font_metrics: FontMetrics,
    /// The glyph runs that make up this text run.
    pub glyphs: Arc<Vec<GlyphRun>>,
}

/// A single series of glyphs within a text run.
#[derive(Clone, Deserialize, Serialize)]
pub struct GlyphRun {
    /// The glyphs.
    pub glyph_store: Arc<GlyphStore>,
    /// The range of characters in the containing run.
    pub range: Range<CharIndex>,
}
