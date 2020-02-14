/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// NOTE: https://www.chromium.org/directwrite-font-proxy has useful
// information for an approach that we'll likely need to take when the
// renderer moves to a sandboxed process.

use crate::font::{FontHandleMethods, FontMetrics, FontTableMethods};
use crate::font::{FontTableTag, FractionalPixel};
use crate::platform::font_template::FontTemplateData;
use crate::platform::windows::font_context::FontContextHandle;
use crate::platform::windows::font_list::font_from_atom;
use crate::text::glyph::GlyphId;
use app_units::Au;
use dwrote::{Font, FontFace, FontFile};
use dwrote::{FontStretch, FontStyle};
use servo_atoms::Atom;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use style::computed_values::font_stretch::T as StyleFontStretch;
use style::computed_values::font_weight::T as StyleFontWeight;
use style::values::computed::font::FontStyle as StyleFontStyle;
use style::values::generics::font::FontStyle as GenericFontStyle;
use style::values::generics::NonNegative;
use style::values::specified::font::FontStretchKeyword;

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

fn make_tag(tag_bytes: &[u8]) -> FontTableTag {
    assert_eq!(tag_bytes.len(), 4);
    unsafe { *(tag_bytes.as_ptr() as *const FontTableTag) }
}

macro_rules! try_lossy(($result:expr) => ($result.map_err(|_| (()))?));

// Given a set of records, figure out the string indices for the family and face
// names.  We want name_id 1 and 2, and we need to use platform_id == 1 and
// language_id == 0 to avoid limitations in the truetype crate.  We *could* just
// do our own parsing here, and use the offset/length data and pull the values out
// ourselves.
fn get_family_face_indices(records: &[truetype::naming_table::Record]) -> Option<(usize, usize)> {
    let mut family_name_index = None;
    let mut face_name_index = None;

    for i in 0..records.len() {
        // the truetype crate can only decode mac platform format names
        if records[i].platform_id != 1 {
            continue;
        }

        if records[i].language_id != 0 {
            continue;
        }

        if records[i].name_id == 1 {
            family_name_index = Some(i);
        } else if records[i].name_id == 2 {
            face_name_index = Some(i);
        }
    }

    if family_name_index.is_some() && face_name_index.is_some() {
        Some((family_name_index.unwrap(), face_name_index.unwrap()))
    } else {
        None
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

impl FontInfo {
    fn new_from_face(face: &FontFace) -> Result<FontInfo, ()> {
        use std::cmp::{max, min};
        use std::io::Cursor;
        use truetype::{NamingTable, Value, WindowsMetrics};

        let name_table_bytes = face.get_font_table(make_tag(b"name"));
        let os2_table_bytes = face.get_font_table(make_tag(b"OS/2"));
        if name_table_bytes.is_none() || os2_table_bytes.is_none() {
            return Err(());
        }

        let mut name_table_cursor = Cursor::new(name_table_bytes.as_ref().unwrap());
        let names = try_lossy!(NamingTable::read(&mut name_table_cursor));
        let (family, face) = match names {
            NamingTable::Format0(ref table) => {
                if let Some((family_index, face_index)) = get_family_face_indices(&table.records) {
                    let strings = table.strings().unwrap();
                    let family = strings[family_index].clone();
                    let face = strings[face_index].clone();
                    (family, face)
                } else {
                    return Err(());
                }
            },
            NamingTable::Format1(ref table) => {
                if let Some((family_index, face_index)) = get_family_face_indices(&table.records) {
                    let strings = table.strings().unwrap();
                    let family = strings[family_index].clone();
                    let face = strings[face_index].clone();
                    (family, face)
                } else {
                    return Err(());
                }
            },
        };

        let mut os2_table_cursor = Cursor::new(os2_table_bytes.as_ref().unwrap());
        let metrics = try_lossy!(WindowsMetrics::read(&mut os2_table_cursor));
        let (weight_val, width_val, italic_bool) = match metrics {
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

        let weight = StyleFontWeight(weight_val as f32);

        let stretch = StyleFontStretch(NonNegative(
            match min(9, max(1, width_val)) {
                1 => FontStretchKeyword::UltraCondensed,
                2 => FontStretchKeyword::ExtraCondensed,
                3 => FontStretchKeyword::Condensed,
                4 => FontStretchKeyword::SemiCondensed,
                5 => FontStretchKeyword::Normal,
                6 => FontStretchKeyword::SemiExpanded,
                7 => FontStretchKeyword::Expanded,
                8 => FontStretchKeyword::ExtraExpanded,
                9 => FontStretchKeyword::UltraExpanded,
                _ => return Err(()),
            }
            .compute(),
        ));

        let style = if italic_bool {
            GenericFontStyle::Italic
        } else {
            GenericFontStyle::Normal
        };

        Ok(FontInfo {
            family_name: family,
            face_name: face,
            weight,
            stretch,
            style,
        })
    }

    fn new_from_font(font: &Font) -> Result<FontInfo, ()> {
        let style = match font.style() {
            FontStyle::Normal => GenericFontStyle::Normal,
            FontStyle::Oblique => GenericFontStyle::Oblique(StyleFontStyle::default_angle()),
            FontStyle::Italic => GenericFontStyle::Italic,
        };
        let weight = StyleFontWeight(font.weight().to_u32() as f32);
        let stretch = StyleFontStretch(NonNegative(
            match font.stretch() {
                FontStretch::Undefined => FontStretchKeyword::Normal,
                FontStretch::UltraCondensed => FontStretchKeyword::UltraCondensed,
                FontStretch::ExtraCondensed => FontStretchKeyword::ExtraCondensed,
                FontStretch::Condensed => FontStretchKeyword::Condensed,
                FontStretch::SemiCondensed => FontStretchKeyword::SemiCondensed,
                FontStretch::Normal => FontStretchKeyword::Normal,
                FontStretch::SemiExpanded => FontStretchKeyword::SemiExpanded,
                FontStretch::Expanded => FontStretchKeyword::Expanded,
                FontStretch::ExtraExpanded => FontStretchKeyword::ExtraExpanded,
                FontStretch::UltraExpanded => FontStretchKeyword::UltraExpanded,
            }
            .compute(),
        ));

        Ok(FontInfo {
            family_name: font.family_name(),
            face_name: font.face_name(),
            style,
            weight,
            stretch,
        })
    }
}

#[derive(Debug)]
pub struct FontHandle {
    font_data: Arc<FontTemplateData>,
    face: Nondebug<FontFace>,
    info: FontInfo,
    em_size: f32,
    du_per_em: f32,
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

impl FontHandle {}

impl FontHandleMethods for FontHandle {
    fn new_from_template(
        _: &FontContextHandle,
        template: Arc<FontTemplateData>,
        pt_size: Option<Au>,
    ) -> Result<Self, ()> {
        let (info, face) = if let Some(ref raw_font) = template.bytes {
            let font_file = FontFile::new_from_data(Arc::new(raw_font.clone()));
            if font_file.is_none() {
                // failed to load raw font
                return Err(());
            }

            let face = font_file
                .unwrap()
                .create_face(0, dwrote::DWRITE_FONT_SIMULATIONS_NONE)
                .map_err(|_| ())?;
            let info = FontInfo::new_from_face(&face)?;
            (info, face)
        } else {
            let font = font_from_atom(&template.identifier);
            let face = font.create_font_face();
            let info = FontInfo::new_from_font(&font)?;
            (info, face)
        };

        let pt_size = pt_size.unwrap_or(au_from_pt(12.));
        let du_per_em = face.metrics().designUnitsPerEm as f32;

        let em_size = pt_size.to_f32_px() / 16.;
        let design_units_per_pixel = du_per_em / 16.;

        let design_units_to_pixels = 1. / design_units_per_pixel;
        let scaled_design_units_to_pixels = em_size / design_units_per_pixel;

        Ok(FontHandle {
            font_data: template.clone(),
            face: Nondebug(face),
            info: info,
            em_size: em_size,
            du_per_em: du_per_em,
            du_to_px: design_units_to_pixels,
            scaled_du_to_px: scaled_design_units_to_pixels,
        })
    }

    fn template(&self) -> Arc<FontTemplateData> {
        self.font_data.clone()
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
        let dm = self.face.metrics();

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

    fn identifier(&self) -> Atom {
        self.font_data.identifier.clone()
    }
}
