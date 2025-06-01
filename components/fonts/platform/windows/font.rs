/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// NOTE: https://www.chromium.org/directwrite-font-proxy has useful
// information for an approach that we'll likely need to take when the
// renderer moves to a sandboxed process.

use std::fmt;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::Arc;

use app_units::Au;
use dwrote::{FontCollection, FontFace, FontFile};
use euclid::default::{Point2D, Rect, Size2D};
use log::{debug, warn};
use style::Zero;
use style::computed_values::font_stretch::T as StyleFontStretch;
use style::computed_values::font_weight::T as StyleFontWeight;
use style::values::computed::font::FontStyle as StyleFontStyle;
use truetype::tables::WindowsMetrics;
use truetype::value::Read;
use webrender_api::FontInstanceFlags;

use super::font_list::LocalFontIdentifier;
use crate::{
    FontData, FontIdentifier, FontMetrics, FontTableMethods, FontTableTag, FontTemplateDescriptor,
    FractionalPixel, GlyphId, PlatformFontMethods, ot_tag,
};

// 1em = 12pt = 16px, assuming 72 points per inch and 96 px per inch
fn pt_to_px(pt: f64) -> f64 {
    pt / 72. * 96.
}
fn em_to_px(em: f64) -> f64 {
    em * 16.
}
fn au_from_em(em: f64) -> Au {
    Au::from_f64_px(em_to_px(em))
}
fn au_from_pt(pt: f64) -> Au {
    Au::from_f64_px(pt_to_px(pt))
}

pub struct FontTable {
    data: Vec<u8>,
}

impl FontTable {
    pub fn wrap(data: &[u8]) -> FontTable {
        FontTable {
            data: data.to_vec(),
        }
    }
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug)]
pub struct PlatformFont {
    face: Nondebug<FontFace>,
    em_size: f32,
    du_to_px: f32,
    scaled_du_to_px: f32,
}

// Based on information from the Skia codebase, it seems that DirectWrite APIs from
// Windows 10 and beyond are thread safe.  If problems arise from this, we can protect the
// platform font with a Mutex.
// See https://source.chromium.org/chromium/chromium/src/+/main:third_party/skia/src/ports/SkScalerContext_win_dw.cpp;l=56;bpv=0;bpt=1.
unsafe impl Sync for PlatformFont {}
unsafe impl Send for PlatformFont {}

struct Nondebug<T>(T);

impl<T> fmt::Debug for Nondebug<T> {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl<T> Deref for Nondebug<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl PlatformFont {
    fn new(font_face: FontFace, pt_size: Option<Au>) -> Result<Self, &'static str> {
        let pt_size = pt_size.unwrap_or(au_from_pt(12.));
        let du_per_em = font_face.metrics().metrics0().designUnitsPerEm as f32;

        let em_size = pt_size.to_f32_px() / 16.;
        let design_units_per_pixel = du_per_em / 16.;

        let design_units_to_pixels = 1. / design_units_per_pixel;
        let scaled_design_units_to_pixels = em_size / design_units_per_pixel;

        Ok(PlatformFont {
            face: Nondebug(font_face),
            em_size,
            du_to_px: design_units_to_pixels,
            scaled_du_to_px: scaled_design_units_to_pixels,
        })
    }
}

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        _font_identifier: FontIdentifier,
        data: &FontData,
        pt_size: Option<Au>,
    ) -> Result<Self, &'static str> {
        let font_face = FontFile::new_from_buffer(Arc::new(data.clone()))
            .ok_or("Could not create FontFile")?
            .create_face(
                0, /* face_index */
                dwrote::DWRITE_FONT_SIMULATIONS_NONE,
            )
            .map_err(|_| "Could not create FontFace")?;
        Self::new(font_face, pt_size)
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let font_face = FontCollection::system()
            .font_from_descriptor(&font_identifier.font_descriptor)
            .ok()
            .flatten()
            .ok_or("Could not create Font from descriptor")?
            .create_font_face();
        Self::new(font_face, pt_size)
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        // We need the font (DWriteFont) in order to be able to query things like
        // the family name, face name, weight, etc.  On Windows 10, the
        // DWriteFontFace3 interface provides this on the FontFace, but that's only
        // available on Win10+.
        //
        // Instead, we do the parsing work using the truetype crate for raw fonts.
        // We're just extracting basic info, so this is sufficient for now.
        //
        // The `dwrote` APIs take SFNT table tags in a reversed byte order, which
        // is why `u32::swap_bytes()` is called here.
        let windows_metrics_bytes = self
            .face
            .get_font_table(u32::swap_bytes(ot_tag!('O', 'S', '/', '2')));
        if windows_metrics_bytes.is_none() {
            warn!("Could not find OS/2 table in font.");
            return FontTemplateDescriptor::default();
        }

        let mut cursor = Cursor::new(windows_metrics_bytes.as_ref().unwrap());
        let Ok(table) = WindowsMetrics::read(&mut cursor) else {
            warn!("Could not read OS/2 table in font.");
            return FontTemplateDescriptor::default();
        };

        let (weight_val, width_val, italic_bool) = match table {
            WindowsMetrics::Version0(ref m) => {
                (m.weight_class, m.width_class, m.selection_flags.0 & 1 == 1)
            },
            WindowsMetrics::Version1(ref m) => {
                (m.weight_class, m.width_class, m.selection_flags.0 & 1 == 1)
            },
            WindowsMetrics::Version2(ref m) |
            WindowsMetrics::Version3(ref m) |
            WindowsMetrics::Version4(ref m) => {
                (m.weight_class, m.width_class, m.selection_flags.0 & 1 == 1)
            },
            WindowsMetrics::Version5(ref m) => {
                (m.weight_class, m.width_class, m.selection_flags.0 & 1 == 1)
            },
        };

        let weight = StyleFontWeight::from_float(weight_val as f32);
        let stretch = match width_val.clamp(1, 9) {
            1 => StyleFontStretch::ULTRA_CONDENSED,
            2 => StyleFontStretch::EXTRA_CONDENSED,
            3 => StyleFontStretch::CONDENSED,
            4 => StyleFontStretch::SEMI_CONDENSED,
            5 => StyleFontStretch::NORMAL,
            6 => StyleFontStretch::SEMI_EXPANDED,
            7 => StyleFontStretch::EXPANDED,
            8 => StyleFontStretch::EXTRA_EXPANDED,
            9 => StyleFontStretch::ULTRA_CONDENSED,
            _ => {
                warn!("Unknown stretch size.");
                StyleFontStretch::NORMAL
            },
        };

        let style = if italic_bool {
            StyleFontStyle::ITALIC
        } else {
            StyleFontStyle::NORMAL
        };

        FontTemplateDescriptor::new(weight, stretch, style)
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

    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        let adjustment = self
            .face
            .get_glyph_pair_kerning_adjustment(first_glyph as u16, second_glyph as u16);

        (adjustment as f32 * self.scaled_du_to_px) as FractionalPixel
    }

    fn metrics(&self) -> FontMetrics {
        let dm = self.face.metrics().metrics0();

        let au_from_du = |du| -> Au { Au::from_f32_px(du as f32 * self.du_to_px) };
        let au_from_du_s = |du| -> Au { Au::from_f32_px(du as f32 * self.scaled_du_to_px) };

        // anything that we calculate and don't just pull out of self.face.metrics
        // is pulled out here for clarity
        let leading = dm.ascent - dm.capHeight;

        let zero_horizontal_advance = self
            .glyph_index('0')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let ic_horizontal_advance = self
            .glyph_index('\u{6C34}')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);

        // TODO: These should be retrieved from the OS/2 table if possible and then
        // fall back to measuring 'x' if that is not available.
        let average_advance = Au::zero();
        let max_advance = Au::zero();

        let space_advance = self
            .glyph_index(' ')
            .and_then(|index| self.glyph_h_advance(index))
            .map(Au::from_f64_px)
            .unwrap_or(average_advance);

        let metrics = FontMetrics {
            underline_size: au_from_du(dm.underlineThickness as i32),
            underline_offset: au_from_du_s(dm.underlinePosition as i32),
            strikeout_size: au_from_du(dm.strikethroughThickness as i32),
            strikeout_offset: au_from_du_s(dm.strikethroughPosition as i32),
            leading: au_from_du_s(leading as i32),
            x_height: au_from_du_s(dm.xHeight as i32),
            em_size: au_from_em(self.em_size as f64),
            ascent: au_from_du_s(dm.ascent as i32),
            descent: au_from_du_s(dm.descent as i32),
            max_advance,
            average_advance,
            line_gap: au_from_du_s((dm.ascent + dm.descent + dm.lineGap as u16) as i32),
            zero_horizontal_advance,
            ic_horizontal_advance,
            space_advance,
        };
        debug!("Font metrics (@{} pt): {:?}", self.em_size * 12., metrics);
        metrics
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        // dwrote (and presumably the Windows APIs) accept a reversed version of the table
        // tag bytes, which means that `u32::swap_bytes` must be called here in order to
        // use a byte order compatible with the rest of Servo.
        self.face
            .get_font_table(u32::swap_bytes(tag))
            .map(|bytes| FontTable { data: bytes })
    }

    fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        FontInstanceFlags::SUBPIXEL_POSITION
    }

    fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        let metrics = self
            .face
            .get_design_glyph_metrics(&[glyph_id as u16], false);
        let metrics = &metrics[0];
        let advance_width = metrics.advanceWidth as f32;
        let advance_height = metrics.advanceHeight as f32;
        let left_side_bearing = metrics.leftSideBearing as f32;
        let right_side_bearing = metrics.rightSideBearing as f32;
        let top_side_bearing = metrics.topSideBearing as f32;
        let bottom_side_bearing = metrics.bottomSideBearing as f32;
        let vertical_origin_y = metrics.verticalOriginY as f32;
        let y_offset = vertical_origin_y + bottom_side_bearing - advance_height;
        let width = advance_width - (left_side_bearing + right_side_bearing);
        let height = advance_height - (top_side_bearing + bottom_side_bearing);

        Rect::new(
            Point2D::new(left_side_bearing, y_offset),
            Size2D::new(width, height),
        )
    }
}
