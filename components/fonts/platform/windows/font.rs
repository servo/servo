/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// NOTE: https://www.chromium.org/directwrite-font-proxy has useful
// information for an approach that we'll likely need to take when the
// renderer moves to a sandboxed process.

use std::ffi::c_void;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use app_units::Au;
use dwrote::{
    DWRITE_FONT_AXIS_VALUE, DWRITE_FONT_SIMULATIONS_BOLD, DWRITE_FONT_SIMULATIONS_NONE,
    FontCollection, FontFace, FontFile, FontSimulations,
};
use euclid::default::{Point2D, Rect, Size2D};
use fonts_traits::{FontData, LocalFontIdentifier};
use log::debug;
use skrifa::font::{Blob, TableFunction};
use skrifa::{Font as SkrifaFont, Tag};
use style::Zero;
use webrender_api::{FontInstanceFlags, FontVariation};
use winapi::shared::minwindef::{BOOL, FALSE};

use crate::font::{FontTableMethods, FractionalPixel};
use crate::{FontIdentifier, FontMetrics, GlyphId, PlatformFontMethods};

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
    variations: Vec<FontVariation>,
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
    fn new(
        font_face: FontFace,
        pt_size: Option<Au>,
        variations: Vec<FontVariation>,
    ) -> Result<Self, &'static str> {
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
            variations,
        })
    }

    fn new_with_variations(
        font_face: FontFace,
        pt_size: Option<Au>,
        variations: &[FontVariation],
    ) -> Result<Self, &'static str> {
        // On FreeType and CoreText platforms, the platform layer is able to read the minimum, maximum,
        // and default values of each axis. This doesn't seem possible here and it seems that Gecko
        // also just sets the value of the axis based on the values from the style as well.
        //
        // dwrote (and presumably the Windows APIs) accept a reversed version of the table
        // tag bytes, which means that `u32::swap_bytes` must be called here in order to
        // use a byte order compatible with the rest of Servo.
        let variations: Vec<_> = variations
            .into_iter()
            .map(|variation| DWRITE_FONT_AXIS_VALUE {
                axisTag: variation.tag.swap_bytes(),
                value: variation.value,
            })
            .collect();

        let Some(font_face) =
            font_face.create_font_face_with_variations(DWRITE_FONT_SIMULATIONS_NONE, &variations)
        else {
            return Err("Could not adapt FontFace to given variations");
        };

        let variations = font_face.variations().unwrap_or_default();
        let variations = variations
            .iter()
            .map(|dwrote_variation| FontVariation {
                tag: dwrote_variation.axisTag.swap_bytes(),
                value: dwrote_variation.value,
            })
            .collect();

        Self::new(font_face, pt_size, variations)
    }

    fn uses_synthetic_bold(&self) -> bool {
        matches!(
            self.face.simulations(),
            FontSimulations::Bold | FontSimulations::BoldOblique
        )
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
            .create_face(0 /* face_index */, DWRITE_FONT_SIMULATIONS_NONE)
            .map_err(|_| "Could not create FontFace")?;
        Self::new(font_face, pt_size, vec![])
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        pt_size: Option<Au>,
    ) -> Result<PlatformFont, &'static str> {
        let mut font_face = FontCollection::system()
            .font_from_descriptor(&font_identifier.font_descriptor)
            .ok()
            .flatten()
            .ok_or("Could not create Font from descriptor")?
            .create_font_face();

        // If DirectWrite created this font face with simulations already applied, we need
        // to strip them here. That way if Servo decides not to apply synthetic bold the
        // face will reflect that.
        if matches!(
            font_face.simulations(),
            FontSimulations::Bold | FontSimulations::BoldOblique
        ) {
            font_face = font_face.create_font_face_with_simulations(DWRITE_FONT_SIMULATIONS_NONE);
        }

        Self::new(font_face, pt_size, vec![])
    }

    fn copy_with_variations(
        self,
        _: &FontIdentifier,
        variations: &[FontVariation],
    ) -> Result<Self, &'static str> {
        Self::new_with_variations(
            self.face.0.clone(),
            Some(Au::from_f32_px(self.em_size * 16.)),
            variations,
        )
    }

    fn copy_with_synthetic_bold(self) -> Result<Self, &'static str> {
        let font_face = self
            .face
            .0
            .create_font_face_with_simulations(DWRITE_FONT_SIMULATIONS_BOLD);
        Self::new(font_face, Some(Au::from_f32_px(self.em_size * 16.)), vec![])
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let Ok(glyphs) = self.face.glyph_indices(&[codepoint as u32]) else {
            return None;
        };
        let Some(glyph) = glyphs.first() else {
            return None;
        };
        if *glyph == 0 {
            return None;
        }
        Some(*glyph as GlyphId)
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        if glyph == 0 {
            return None;
        }
        let Ok(metrics) = self.face.design_glyph_metrics(&[glyph as u16], false) else {
            return None;
        };
        let Some(glyph_metric) = metrics.first() else {
            return None;
        };
        Some((glyph_metric.advanceWidth as f32 * self.scaled_du_to_px) as FractionalPixel)
    }

    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        let adjustment = self
            .face
            .glyph_pair_kerning_adjustment(first_glyph as u16, second_glyph as u16)
            .unwrap_or_default();

        (adjustment as f32 * self.scaled_du_to_px) as FractionalPixel
    }

    fn metrics(&self) -> FontMetrics {
        let dm = self.face.metrics().metrics0();

        let au_from_du = |du| -> Au { Au::from_f32_px(du as f32 * self.du_to_px) };
        let au_from_du_s = |du| -> Au { Au::from_f32_px(du as f32 * self.scaled_du_to_px) };

        // anything that we calculate and don't just pull out of self.face.metrics
        // is pulled out here for clarity
        // Values may be negative, so we convert to signed integers, see
        // <https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ns-dwrite-dwrite_font_metrics>
        let leading = i32::from(dm.lineGap);
        let line_height = i32::from(dm.ascent) + i32::from(dm.descent) + leading;

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
            leading: au_from_du_s(leading),
            x_height: au_from_du_s(dm.xHeight as i32),
            em_size: au_from_em(self.em_size as f64),
            ascent: au_from_du_s(dm.ascent as i32),
            descent: au_from_du_s(dm.descent as i32),
            max_advance,
            average_advance,
            line_gap: au_from_du_s(line_height),
            zero_horizontal_advance,
            ic_horizontal_advance,
            space_advance,
        };
        debug!("Font metrics (@{} pt): {:?}", self.em_size * 12., metrics);
        metrics
    }

    fn table_for_tag(&self, tag: Tag) -> Option<FontTable> {
        // dwrote (and presumably the Windows APIs) accept a reversed version of the table
        // tag bytes, which means that `u32::swap_bytes` must be called here in order to
        // use a byte order compatible with the rest of Servo.
        self.face
            .font_table(u32::from_be_bytes(tag.to_be_bytes()).swap_bytes())
            .ok()
            .flatten()
            .map(|bytes| FontTable { data: bytes })
    }

    fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        let mut flags = FontInstanceFlags::SUBPIXEL_POSITION;

        // TODO: Add support for synthetic italics.
        // <https://github.com/servo/servo/issues/39637>
        if self.uses_synthetic_bold() {
            flags |= FontInstanceFlags::SYNTHETIC_BOLD;
        }

        flags
    }

    fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        let Ok(metrics) = self.face.design_glyph_metrics(&[glyph_id as u16], false) else {
            return Rect::zero();
        };
        let Some(metrics) = metrics.first() else {
            return Rect::zero();
        };
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

    fn variations(&self) -> &[FontVariation] {
        &self.variations
    }

    fn create_skrifa_font(&self) -> Option<SkrifaFont> {
        let font = ThreadSafeDirectWriteFontFace::new(self.face.0.clone());
        let table_function = TableFunction::new(Arc::new(move |tag: Tag| -> Option<Blob> {
            font.table_data_blob(tag)
        }));
        SkrifaFont::new(
            table_function,
            0, /* This value is ignored, for a table function */
        )
    }
}

/// A wrapper struct around [`FontFace`] which is responsible for allowing
/// access to table data across threads.
struct ThreadSafeDirectWriteFontFace {
    face: FontFace,
}

// SAFETY: Table data is immutable and we assume that DirectWrite handles are thread safe.
// See the comment about this on PlatformFont above.
unsafe impl Sync for ThreadSafeDirectWriteFontFace {}
unsafe impl Send for ThreadSafeDirectWriteFontFace {}

impl ThreadSafeDirectWriteFontFace {
    fn new(face: FontFace) -> Self {
        Self { face }
    }

    fn table_data_blob(&self, tag: Tag) -> Option<Blob> {
        let direct_write_face = unsafe { self.face.as_ptr() };
        if direct_write_face.is_null() {
            return None;
        }

        let direct_write_face = unsafe { &*direct_write_face };
        let direct_write_tag = u32::from_be_bytes(tag.to_be_bytes()).swap_bytes();
        let mut table_data_ptr: *const u8 = std::ptr::null_mut();
        let mut table_length: u32 = 0;
        let mut table_context: *mut c_void = std::ptr::null_mut();
        let mut exists: BOOL = FALSE;

        let hr = unsafe {
            direct_write_face.TryGetFontTable(
                direct_write_tag,
                &mut table_data_ptr as *mut *const _ as *mut *const c_void,
                &mut table_length,
                &mut table_context,
                &mut exists,
            )
        };

        if hr != 0 || exists == 0 {
            return None;
        }

        let table = ThreadSafeDirectWriteFontTable {
            face: self.face.clone(),
            context: table_context,
            data: table_data_ptr,
            length: table_length as usize,
        };
        if table.data.is_null() || table.length == 0 {
            return None;
        }

        Some(Blob::Shared(Arc::new(table)))
    }
}

struct ThreadSafeDirectWriteFontTable {
    face: FontFace,
    context: *mut c_void,
    data: *const u8,
    length: usize,
}

// SAFETY: The slice returned from DirectWrite must not be mutated while the slice is
// alive. Data returned from DirectWrite is immutable.
unsafe impl Send for ThreadSafeDirectWriteFontTable {}
unsafe impl Sync for ThreadSafeDirectWriteFontTable {}

impl AsRef<[u8]> for ThreadSafeDirectWriteFontTable {
    fn as_ref(&self) -> &[u8] {
        if self.data.is_null() {
            return &[];
        }
        // SAFETY: The slice returned from DirectWrite must not be mutated while the slice
        // is alive. Data returned from DirectWrite font tables is immutable.
        unsafe { std::slice::from_raw_parts(self.data, self.length) }
    }
}

impl Drop for ThreadSafeDirectWriteFontTable {
    fn drop(&mut self) {
        let direct_write_face = unsafe { self.face.as_ptr() };
        assert!(!direct_write_face.is_null());
        let direct_write_face = unsafe { &*direct_write_face };
        unsafe { direct_write_face.ReleaseFontTable(self.context) };
    }
}
