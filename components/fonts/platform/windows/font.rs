/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// NOTE: https://www.chromium.org/directwrite-font-proxy has useful
// information for an approach that we'll likely need to take when the
// renderer moves to a sandboxed process.

use std::cell::RefCell;
use std::ffi::c_void;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use app_units::Au;
use dwrote::{
    DWRITE_FONT_AXIS_VALUE, DWRITE_FONT_SIMULATIONS, DWRITE_FONT_SIMULATIONS_BOLD,
    DWRITE_FONT_SIMULATIONS_NONE, FontCollection, FontFace, FontFile,
};
use euclid::default::{Point2D, Rect, Size2D};
use fonts_traits::LocalFontIdentifier;
use log::debug;
use read_fonts::TableProvider;
use skrifa::Tag;
use style::Zero;
use webrender_api::{FontInstanceFlags, FontVariation};
use winapi::shared::minwindef::{BOOL, FALSE};

use crate::{
    FontData, FontIdentifier, FontMetrics, FontTableMethods, FontTemplateDescriptor,
    FractionalPixel, GlyphId, PlatformFontMethods,
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

fn synthetic_bold_to_simulations(synthetic_bold: bool) -> DWRITE_FONT_SIMULATIONS {
    match synthetic_bold {
        true => DWRITE_FONT_SIMULATIONS_BOLD,
        false => DWRITE_FONT_SIMULATIONS_NONE,
    }
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
    synthetic_bold: bool,
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
        synthetic_bold: bool,
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
            synthetic_bold,
        })
    }

    fn new_with_variations(
        font_face: FontFace,
        pt_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<Self, &'static str> {
        if variations.is_empty() {
            return Self::new(font_face, pt_size, vec![], synthetic_bold);
        }

        let simulations = synthetic_bold_to_simulations(synthetic_bold);

        // On FreeType and CoreText platforms, the platform layer is able to read the minimum, maxmimum,
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

        let Some(font_face) = font_face.create_font_face_with_variations(simulations, &variations)
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

        Self::new(font_face, pt_size, variations, synthetic_bold)
    }
}

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        _font_identifier: FontIdentifier,
        data: &FontData,
        pt_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<Self, &'static str> {
        let simulations = synthetic_bold_to_simulations(synthetic_bold);
        let font_face = FontFile::new_from_buffer(Arc::new(data.clone()))
            .ok_or("Could not create FontFile")?
            .create_face(0 /* face_index */, simulations)
            .map_err(|_| "Could not create FontFace")?;
        Self::new_with_variations(font_face, pt_size, variations, synthetic_bold)
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        pt_size: Option<Au>,
        variations: &[FontVariation],
        synthetic_bold: bool,
    ) -> Result<PlatformFont, &'static str> {
        let font_face = FontCollection::system()
            .font_from_descriptor(&font_identifier.font_descriptor)
            .ok()
            .flatten()
            .ok_or("Could not create Font from descriptor")?
            .create_font_face();
        Self::new_with_variations(font_face, pt_size, variations, synthetic_bold)
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        DirectWriteTableProvider::new(self)
            .os2()
            .as_ref()
            .map(Self::descriptor_from_os2_table)
            .unwrap_or_default()
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

        if self.synthetic_bold {
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
}

/// A wrapper struct around [`PlatformFont`] which is responsible for
/// implementing [`TableProvider`] and cleaning up any font table contexts from
/// DirectWrite when the struct is dropped.
struct DirectWriteTableProvider<'platform_font> {
    platform_font: &'platform_font PlatformFont,
    contexts: RefCell<Vec<*mut c_void>>,
}

impl<'platform_font> DirectWriteTableProvider<'platform_font> {
    fn new(platform_font: &'platform_font PlatformFont) -> Self {
        Self {
            platform_font,
            contexts: Default::default(),
        }
    }
}

impl Drop for DirectWriteTableProvider<'_> {
    fn drop(&mut self) {
        let direct_write_face = unsafe { self.platform_font.face.as_ptr() };
        assert!(!direct_write_face.is_null());

        let direct_write_face = unsafe { &*direct_write_face };
        for context in self.contexts.borrow_mut().drain(..) {
            unsafe { direct_write_face.ReleaseFontTable(context) };
        }
    }
}

impl<'platform_font> TableProvider<'platform_font> for DirectWriteTableProvider<'platform_font> {
    fn data_for_tag(&self, tag: Tag) -> Option<read_fonts::FontData<'platform_font>> {
        let direct_write_face = unsafe { self.platform_font.face.as_ptr() };
        if direct_write_face.is_null() {
            return None;
        }

        let direct_write_face = unsafe { &*direct_write_face };
        let direct_write_tag = u32::from_be_bytes(tag.to_be_bytes()).swap_bytes();
        let mut table_data_ptr: *const u8 = std::ptr::null_mut();
        let mut table_size: u32 = 0;
        let mut table_context: *mut c_void = std::ptr::null_mut();
        let mut exists: BOOL = FALSE;

        let hr = unsafe {
            direct_write_face.TryGetFontTable(
                direct_write_tag,
                &mut table_data_ptr as *mut *const _ as *mut *const c_void,
                &mut table_size,
                &mut table_context,
                &mut exists,
            )
        };

        if hr != 0 || exists == 0 {
            return None;
        }

        self.contexts.borrow_mut().push(table_context);

        if table_data_ptr.is_null() || table_size == 0 {
            return None;
        }

        let bytes = unsafe { std::slice::from_raw_parts(table_data_ptr, table_size as usize) };
        Some(read_fonts::FontData::new(bytes))
    }
}
