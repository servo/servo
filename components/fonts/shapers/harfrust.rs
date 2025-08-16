use app_units::Au;
use euclid::default::Point2D;
use harfrust::{
    Feature, FontRef as HarfRustFontRef, GlyphBuffer, Script, ShaperData, Tag, UnicodeBuffer,
};
use num_traits::Zero as _;
use smallvec::SmallVec;

use super::{ShapedGlyphEntry, THarfShapedGlyphData, THarfShaper, unicode_to_hb_script};
use crate::{Font, FontData, ShapingFlags, fixed_to_float};

fn au_from_fixed_glyph_info(i: i32) -> Au {
    Au::from_f64_px(fixed_to_float(16, i))
}

pub struct ShapedGlyphData {
    data: GlyphBuffer,
}

impl THarfShapedGlyphData for ShapedGlyphData {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn byte_offset_of_glyph(&self, i: usize) -> usize {
        self.data.glyph_infos()[i].cluster as usize
    }

    fn entry_for_glyph(&self, i: usize, y_pos: &mut app_units::Au) -> super::ShapedGlyphEntry {
        let glyph_info_i = self.data.glyph_infos()[i];
        let pos_info_i = self.data.glyph_positions()[i];
        let x_offset = au_from_fixed_glyph_info(pos_info_i.x_offset);
        let y_offset = au_from_fixed_glyph_info(pos_info_i.y_offset);
        let x_advance = au_from_fixed_glyph_info(pos_info_i.x_advance);
        let y_advance = au_from_fixed_glyph_info(pos_info_i.y_advance);

        let offset = if x_offset.is_zero() && y_offset.is_zero() && y_advance.is_zero() {
            None
        } else {
            // adjust the pen..
            if y_advance > Au::zero() {
                *y_pos -= y_advance;
            }

            Some(Point2D::new(x_offset, *y_pos - y_offset))
        };

        ShapedGlyphEntry {
            codepoint: glyph_info_i.glyph_id,
            advance: x_advance,
            offset,
        }
    }
}

pub struct Shaper {
    font: *const Font,
    /// The raw byte data of the font
    font_data: FontData,
    /// The index of a font in it's collection (.ttc)
    /// If the font file is not a collection then this is 0
    font_index: u32,
}

// `Font` and `FontData` are both threadsafe, so we can make the data structures here as thread-safe as well.
#[allow(unsafe_code)]
unsafe impl Sync for Shaper {}
#[allow(unsafe_code)]
unsafe impl Send for Shaper {}

impl Shaper {
    pub(crate) fn new(font: &Font) -> Self {
        // TODO: handle scaling
        // Set points-per-em. if zero, performs no hinting in that directi
        // let pt_size = (*font).descriptor.pt_size.to_f64_px();

        let font_data = font.data().clone();
        let font_index = font.identifier().index();
        Self {
            font: font as *const Font,
            font_data,
            font_index,
        }
    }
}

impl THarfShaper for Shaper {
    type ShapedGlyphData = ShapedGlyphData;

    fn shape_text(&self, text: &str, options: &crate::ShapingOptions) -> ShapedGlyphData {
        let mut buffer = UnicodeBuffer::new();

        // Set direction
        buffer.set_direction(if options.flags.contains(ShapingFlags::RTL_FLAG) {
            harfrust::Direction::RightToLeft
        } else {
            harfrust::Direction::LeftToRight
        });

        // Set script
        let script =
            Script::from_iso15924_tag(Tag::from_u32(unicode_to_hb_script(options.script))).unwrap();
        buffer.set_script(script);

        // Push text
        buffer.push_str(text);

        // Features
        let mut features = SmallVec::<[Feature; 2]>::new();
        if options
            .flags
            .contains(ShapingFlags::IGNORE_LIGATURES_SHAPING_FLAG)
        {
            features.push(Feature::new(Tag::new(b"liga"), 0, ..));
        }
        if options
            .flags
            .contains(ShapingFlags::DISABLE_KERNING_SHAPING_FLAG)
        {
            features.push(Feature::new(Tag::new(b"kern"), 0, ..));
        }

        let hr_font =
            HarfRustFontRef::from_index(self.font_data.as_ref(), self.font_index).unwrap();
        let shaper_data = ShaperData::new(&hr_font);

        // TODO: handle font variations
        // let variations: Vec<Variation> = Vec::new();
        // let instance_data = ShaperInstance::from_variations(&hr_font, &variations);

        let shaper = shaper_data
            .shaper(&hr_font)
            // Set the instance
            // .instance(Some(&instance_data))
            .build();
        let glyph_buffer = shaper.shape(buffer, &features);

        ShapedGlyphData { data: glyph_buffer }
    }

    #[allow(unsafe_code)]
    fn font(&self) -> &Font {
        // SAFETY: the font actually owns this shaper so it cannot have been dropped
        unsafe { &(*self.font) }
    }

    fn baseline(&self) -> Option<crate::FontBaseline> {
        // TODO: Implement baseline extraction
        None
    }
}
