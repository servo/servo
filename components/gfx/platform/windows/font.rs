/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// NOTE: https://www.chromium.org/directwrite-font-proxy has useful
// information for an approach that we'll likely need to take when the
// renderer moves to a sandboxed process.

use app_units::Au;
use dwrote::{Font, FontFace};
use dwrote::{FontWeight, FontStretch, FontStyle};
use font::{FontHandleMethods, FontMetrics, FontTableMethods};
use font::{FontTableTag, FractionalPixel};
use platform::font_template::FontTemplateData;
use platform::windows::font_context::FontContextHandle;
use platform::windows::font_list::font_from_atom;
use std::sync::Arc;
use style::computed_values::{font_stretch, font_weight};
use text::glyph::GlyphId;

// 1em = 12pt = 16px, assuming 72 points per inch and 96 px per inch
fn pt_to_px(pt: f64) -> f64 { pt / 72. * 96. }
fn em_to_px(em: f64) -> f64 { em * 16. }
fn au_from_em(em: f64) -> Au { Au::from_f64_px(em_to_px(em)) }
fn au_from_pt(pt: f64) -> Au { Au::from_f64_px(pt_to_px(pt)) }

pub struct FontTable {
    data: Vec<u8>,
}

impl FontTable {
    pub fn wrap(data: &[u8]) -> FontTable {
        FontTable { data: data.to_vec() }
    }
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug)]
pub struct FontHandle {
    font_data: Arc<FontTemplateData>,
    font: Font,
    face: FontFace,
    em_size: f32,
    du_per_em: f32,
    du_to_px: f32,
    scaled_du_to_px: f32,
}

impl FontHandle {
}

impl FontHandleMethods for FontHandle {
    fn new_from_template(_: &FontContextHandle, template: Arc<FontTemplateData>, pt_size: Option<Au>)
                         -> Result<Self, ()>
    {
        if let Some(_) = template.bytes {
            // FIXME we should load from template.bytes
            Err(())
        } else {
            let font = font_from_atom(&template.identifier);
            let face = font.create_font_face();

            let pt_size = pt_size.unwrap_or(au_from_pt(12.));
            let du_per_em = face.metrics().designUnitsPerEm as f32;

            let em_size = pt_size.to_f32_px() / 16.;
            let design_units_per_pixel = du_per_em / 16.;

            let design_units_to_pixels = 1. / design_units_per_pixel;
            let scaled_design_units_to_pixels = em_size / design_units_per_pixel;

            Ok(FontHandle {
                font_data: template.clone(),
                font: font,
                face: face,
                em_size: em_size,
                du_per_em: du_per_em,
                du_to_px: design_units_to_pixels,
                scaled_du_to_px: scaled_design_units_to_pixels,
            })
        }
    }

    fn template(&self) -> Arc<FontTemplateData> {
        self.font_data.clone()
    }

    fn family_name(&self) -> String {
        self.font.family_name()
    }

    fn face_name(&self) -> String {
        self.font.face_name()
    }

    fn is_italic(&self) -> bool {
        match self.font.style() {
            FontStyle::Normal => false,
            FontStyle::Oblique | FontStyle::Italic => true,
        }
    }

    fn boldness(&self) -> font_weight::T {
        match self.font.weight() {
            FontWeight::Thin => font_weight::T::Weight100,
            FontWeight::ExtraLight => font_weight::T::Weight200,
            FontWeight::Light => font_weight::T::Weight300,
            // slightly lighter gray
            FontWeight::SemiLight => font_weight::T::Weight300,
            FontWeight::Regular => font_weight::T::Weight400,
            FontWeight::Medium => font_weight::T::Weight500,
            FontWeight::SemiBold => font_weight::T::Weight600,
            FontWeight::Bold => font_weight::T::Weight700,
            FontWeight::ExtraBold => font_weight::T::Weight800,
            FontWeight::Black => font_weight::T::Weight900,
            // slightly blacker black
            FontWeight::ExtraBlack => font_weight::T::Weight900,
        }
    }

    fn stretchiness(&self) -> font_stretch::T {
        match self.font.stretch() {
            FontStretch::Undefined => font_stretch::T::normal,
            FontStretch::UltraCondensed => font_stretch::T::ultra_condensed,
            FontStretch::ExtraCondensed => font_stretch::T::extra_condensed,
            FontStretch::Condensed => font_stretch::T::condensed,
            FontStretch::SemiCondensed => font_stretch::T::semi_condensed,
            FontStretch::Normal => font_stretch::T::normal,
            FontStretch::SemiExpanded => font_stretch::T::semi_expanded,
            FontStretch::Expanded => font_stretch::T::expanded,
            FontStretch::ExtraExpanded => font_stretch::T::extra_expanded,
            FontStretch::UltraExpanded => font_stretch::T::ultra_expanded,
        }
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let glyph = self.face.get_glyph_indices(&[codepoint as u32])[0];
        if glyph == 0 {
            return None;
        }
        Some(glyph as GlyphId)
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        if glyph == 0 {
            return None;
        }

        let gm = self.face.get_design_glyph_metrics(&[glyph as u16], false)[0];
        let f = (gm.advanceWidth as f32 * self.scaled_du_to_px) as FractionalPixel;

        Some(f)
    }

    /// Can this font do basic horizontal LTR shaping without Harfbuzz?
    fn can_do_fast_shaping(&self) -> bool {
        // TODO copy CachedKernTable from the MacOS X implementation to
        // somehwere global and use it here.  We could also implement the
        // IDirectWriteFontFace1 interface and use the glyph kerning pair
        // methods there.
        false
    }

    fn glyph_h_kerning(&self, _: GlyphId, _: GlyphId) -> FractionalPixel {
        0.0
    }

    fn metrics(&self) -> FontMetrics {
        let dm = self.face.metrics();

        let au_from_du = |du| -> Au { Au::from_f32_px(du as f32 * self.du_to_px) };
        let au_from_du_s = |du| -> Au { Au:: from_f32_px(du as f32 * self.scaled_du_to_px) };

        // anything that we calculate and don't just pull out of self.face.metrics
        // is pulled out here for clarity
        let leading = dm.ascent - dm.capHeight;

        let metrics = FontMetrics {
            underline_size:   au_from_du(dm.underlineThickness as i32),
            underline_offset: au_from_du_s(dm.underlinePosition as i32),
            strikeout_size:   au_from_du(dm.strikethroughThickness as i32),
            strikeout_offset: au_from_du_s(dm.strikethroughPosition as i32),
            leading:          au_from_du_s(leading as i32),
            x_height:         au_from_du_s(dm.xHeight as i32),
            em_size:          au_from_em(self.em_size as f64),
            ascent:           au_from_du_s(dm.ascent as i32),
            descent:          au_from_du_s(dm.descent as i32),
            max_advance:      au_from_pt(0.0), // FIXME
            average_advance:  au_from_pt(0.0), // FIXME
            line_gap:         au_from_du(dm.lineGap as i32),
        };
        debug!("Font metrics (@{} pt): {:?}", self.em_size * 12., metrics);
        metrics
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        self.face.get_font_table(tag).map(|bytes| FontTable { data: bytes })
    }
}
