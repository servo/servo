/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// NOTE: https://www.chromium.org/directwrite-font-proxy has useful
// information for an approach that we'll likely need to take when the
// renderer moves to a sandboxed process.

use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use app_units::Au;
use dwrote::{FontFace, FontFile};
use log::debug;
use style::computed_values::font_stretch::T as StyleFontStretch;
use style::computed_values::font_weight::T as StyleFontWeight;
use style::values::computed::font::FontStyle as StyleFontStyle;
use style::values::specified::font::FontStretchKeyword;

use crate::font::{
    FontMetrics, FontTableMethods, FontTableTag, FractionalPixel, PlatformFontMethods,
};
use crate::font_cache_thread::FontIdentifier;
use crate::text::glyph::GlyphId;

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

// We need the font (DWriteFont) in order to be able to query things like
// the family name, face name, weight, etc.  On Windows 10, the
// DWriteFontFace3 interface provides this on the FontFace, but that's only
// available on Win10+.
//
// Instead, we do the parsing work using the truetype crate for raw fonts.
// We're just extracting basic info, so this is sufficient for now.

#[derive(Debug)]
struct FontInfo {
    family_name: String,
    face_name: String,
    weight: StyleFontWeight,
    stretch: StyleFontStretch,
    style: StyleFontStyle,
}

#[macro_export]
/// Packs the components of a font tag name into 32 bytes, while respecting the
/// necessary Rust 4-byte alignment for pointers. This is similar to
/// [`crate::ot_tag`], but the bytes are reversed.
macro_rules! font_tag {
    ($t1:expr, $t2:expr, $t3:expr, $t4:expr) => {
        (($t4 as u32) << 24) | (($t3 as u32) << 16) | (($t2 as u32) << 8) | ($t1 as u32)
    };
}

impl FontInfo {
    fn new_from_face(face: &FontFace) -> Result<FontInfo, &'static str> {
        use std::cmp::{max, min};
        use std::collections::HashMap;
        use std::io::Cursor;

        use truetype::tables::names::{NameID, Names};
        use truetype::tables::WindowsMetrics;
        use truetype::value::Read;

        let name_tag = font_tag!('n', 'a', 'm', 'e');
        let os2_tag = font_tag!('O', 'S', '/', '2');
        let names_bytes = face.get_font_table(name_tag);
        let windows_metrics_bytes = face.get_font_table(os2_tag);
        if names_bytes.is_none() || windows_metrics_bytes.is_none() {
            return Err("No 'name' or 'OS/2' tables");
        }

        let mut cursor = Cursor::new(names_bytes.as_ref().unwrap());
        let table = Names::read(&mut cursor).map_err(|_| "Could not read 'name' table")?;
        let language_tags = table.language_tags().collect::<Vec<_>>();
        let mut names = table
            .iter()
            .filter(|((_, _, language_id, _), value)| {
                value.is_some() &&
                    language_id
                        .tag(&language_tags)
                        .map_or(false, |tag| tag.starts_with("en"))
            })
            .map(|((_, _, _, name_id), value)| (name_id, value.unwrap()))
            .collect::<HashMap<_, _>>();
        let family = match names.remove(&NameID::FontFamilyName) {
            Some(family) => family,
            _ => return Err("Could not find family"),
        };
        let face = match names.remove(&NameID::FontSubfamilyName) {
            Some(face) => face,
            _ => return Err("Could not find subfamily"),
        };

        let mut cursor = Cursor::new(windows_metrics_bytes.as_ref().unwrap());
        let table = WindowsMetrics::read(&mut cursor).map_err(|_| "Could not read OS/2 table")?;
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

        let stretch = match min(9, max(1, width_val)) {
            1 => FontStretchKeyword::UltraCondensed,
            2 => FontStretchKeyword::ExtraCondensed,
            3 => FontStretchKeyword::Condensed,
            4 => FontStretchKeyword::SemiCondensed,
            5 => FontStretchKeyword::Normal,
            6 => FontStretchKeyword::SemiExpanded,
            7 => FontStretchKeyword::Expanded,
            8 => FontStretchKeyword::ExtraExpanded,
            9 => FontStretchKeyword::UltraExpanded,
            _ => return Err("Unknown stretch size"),
        }
        .compute();

        let style = if italic_bool {
            StyleFontStyle::ITALIC
        } else {
            StyleFontStyle::NORMAL
        };

        Ok(FontInfo {
            family_name: family,
            face_name: face,
            weight,
            stretch,
            style,
        })
    }
}

#[derive(Debug)]
pub struct PlatformFont {
    face: Nondebug<FontFace>,
    /// A reference to this data used to create this [`PlatformFont`], ensuring the
    /// data stays alive of the lifetime of this struct.
    _data: Arc<Vec<u8>>,
    info: FontInfo,
    em_size: f32,
    du_to_px: f32,
    scaled_du_to_px: f32,
}

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

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        _font_identifier: FontIdentifier,
        data: Arc<Vec<u8>>,
        face_index: u32,
        pt_size: Option<Au>,
    ) -> Result<Self, &'static str> {
        let font_file = FontFile::new_from_data(data.clone()).ok_or("Could not create FontFile")?;
        let face = font_file
            .create_face(face_index, dwrote::DWRITE_FONT_SIMULATIONS_NONE)
            .map_err(|_| "Could not create FontFace")?;
        let info = FontInfo::new_from_face(&face)?;

        let pt_size = pt_size.unwrap_or(au_from_pt(12.));
        let du_per_em = face.metrics().metrics0().designUnitsPerEm as f32;

        let em_size = pt_size.to_f32_px() / 16.;
        let design_units_per_pixel = du_per_em / 16.;

        let design_units_to_pixels = 1. / design_units_per_pixel;
        let scaled_design_units_to_pixels = em_size / design_units_per_pixel;

        Ok(PlatformFont {
            face: Nondebug(face),
            _data: data,
            info,
            em_size,
            du_to_px: design_units_to_pixels,
            scaled_du_to_px: scaled_design_units_to_pixels,
        })
    }

    fn family_name(&self) -> Option<String> {
        Some(self.info.family_name.clone())
    }

    fn face_name(&self) -> Option<String> {
        Some(self.info.face_name.clone())
    }

    fn style(&self) -> StyleFontStyle {
        self.info.style
    }

    fn boldness(&self) -> StyleFontWeight {
        self.info.weight
    }

    fn stretchiness(&self) -> StyleFontStretch {
        self.info.stretch
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
        let dm = self.face.metrics().metrics0();

        let au_from_du = |du| -> Au { Au::from_f32_px(du as f32 * self.du_to_px) };
        let au_from_du_s = |du| -> Au { Au::from_f32_px(du as f32 * self.scaled_du_to_px) };

        // anything that we calculate and don't just pull out of self.face.metrics
        // is pulled out here for clarity
        let leading = dm.ascent - dm.capHeight;

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
            max_advance: au_from_pt(0.0),     // FIXME
            average_advance: au_from_pt(0.0), // FIXME
            line_gap: au_from_du_s((dm.ascent + dm.descent + dm.lineGap as u16) as i32),
        };
        debug!("Font metrics (@{} pt): {:?}", self.em_size * 12., metrics);
        metrics
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        self.face
            .get_font_table(tag)
            .map(|bytes| FontTable { data: bytes })
    }
}
