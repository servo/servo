/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// NOTE: https://www.chromium.org/directwrite-font-proxy has useful
// information for an approach that we'll likely need to take when the
// renderer moves to a sandboxed process.

use app_units::Au;
use dwrote;
use dwrote::{Font, FontFace, FontFile};
use dwrote::{FontWeight, FontStretch, FontStyle};
use font::{FontHandleMethods, FontMetrics, FontTableMethods};
use font::{FontTableTag, FractionalPixel};
use platform::font_template::FontTemplateData;
use platform::windows::font_context::FontContextHandle;
use platform::windows::font_list::font_from_atom;
use std::sync::Arc;
use style::computed_values::{font_stretch, font_weight};
use text::glyph::GlyphId;
use truetype;

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

fn make_tag(tag_bytes: &[u8]) -> FontTableTag {
    assert_eq!(tag_bytes.len(), 4);
    unsafe { *(tag_bytes.as_ptr() as *const FontTableTag) }
}

macro_rules! try_lossy(($result:expr) => (try!($result.map_err(|_| (())))));

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
    weight: font_weight::T,
    stretch: font_stretch::T,
    style: FontStyle,
}

impl FontInfo {
    fn new_from_face(face: &FontFace) -> Result<FontInfo, ()> {
        use std::cmp::{min, max};
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
                    ((family, face))
                } else {
                    return Err(());
                }
            },
            NamingTable::Format1(ref table) => {
                if let Some((family_index, face_index)) = get_family_face_indices(&table.records) {
                    let strings = table.strings().unwrap();
                    let family = strings[family_index].clone();
                    let face = strings[face_index].clone();
                    ((family, face))
                } else {
                    return Err(());
                }
            }
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

        let weight = match min(9, max(1, weight_val / 100)) {
            1 => font_weight::T::Weight100,
            2 => font_weight::T::Weight200,
            3 => font_weight::T::Weight300,
            4 => font_weight::T::Weight400,
            5 => font_weight::T::Weight500,
            6 => font_weight::T::Weight600,
            7 => font_weight::T::Weight700,
            8 => font_weight::T::Weight800,
            9 => font_weight::T::Weight900,
            _ => return Err(()),
        };

        let stretch = match min(9, max(1, width_val)) {
            1 => font_stretch::T::ultra_condensed,
            2 => font_stretch::T::extra_condensed,
            3 => font_stretch::T::condensed,
            4 => font_stretch::T::semi_condensed,
            5 => font_stretch::T::normal,
            6 => font_stretch::T::semi_expanded,
            7 => font_stretch::T::expanded,
            8 => font_stretch::T::extra_expanded,
            9 => font_stretch::T::ultra_expanded,
            _ => return Err(()),
        };

        let style = if italic_bool {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };

        Ok(FontInfo {
            family_name: family,
            face_name: face,
            weight: weight,
            stretch: stretch,
            style: style,
        })
    }

    fn new_from_font(font: &Font) -> Result<FontInfo, ()> {
        let style = font.style();
        let weight = match font.weight() {
            FontWeight::Thin => font_weight::T::Weight100,
            FontWeight::ExtraLight => font_weight::T::Weight200,
            FontWeight::Light => font_weight::T::Weight300,
            // slightly grayer gray
            FontWeight::SemiLight => font_weight::T::Weight300,
            FontWeight::Regular => font_weight::T::Weight400,
            FontWeight::Medium => font_weight::T::Weight500,
            FontWeight::SemiBold => font_weight::T::Weight600,
            FontWeight::Bold => font_weight::T::Weight700,
            FontWeight::ExtraBold => font_weight::T::Weight800,
            FontWeight::Black => font_weight::T::Weight900,
            // slightly blacker black
            FontWeight::ExtraBlack => font_weight::T::Weight900,
        };
        let stretch = match font.stretch() {
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
        };

        Ok(FontInfo {
            family_name: font.family_name(),
            face_name: font.face_name(),
            style: style,
            weight: weight,
            stretch: stretch,
        })
    }
}

#[derive(Debug)]
pub struct FontHandle {
    font_data: Arc<FontTemplateData>,
    face: FontFace,
    info: FontInfo,
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
        let (info, face) = if let Some(ref raw_font) = template.bytes {
            let font_file = FontFile::new_from_data(&raw_font);
            if font_file.is_none() {
                // failed to load raw font
                return Err(());
            }

            let face = font_file.unwrap().create_face(0, dwrote::DWRITE_FONT_SIMULATIONS_NONE);
            let info = try!(FontInfo::new_from_face(&face));
            (info, face)
        } else {
            let font = font_from_atom(&template.identifier);
            let face = font.create_font_face();
            let info = try!(FontInfo::new_from_font(&font));
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
            face: face,
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

    fn family_name(&self) -> String {
        self.info.family_name.clone()
    }

    fn face_name(&self) -> Option<String> {
        Some(self.info.face_name.clone())
    }

    fn is_italic(&self) -> bool {
        match self.info.style {
            FontStyle::Normal => false,
            FontStyle::Oblique | FontStyle::Italic => true,
        }
    }

    fn boldness(&self) -> font_weight::T {
        self.info.weight
    }

    fn stretchiness(&self) -> font_stretch::T {
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
            line_gap:         au_from_du_s((dm.ascent + dm.descent + dm.lineGap as u16) as i32),
        };
        debug!("Font metrics (@{} pt): {:?}", self.em_size * 12., metrics);
        metrics
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        self.face.get_font_table(tag).map(|bytes| FontTable { data: bytes })
    }
}
