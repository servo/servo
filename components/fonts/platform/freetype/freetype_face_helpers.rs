/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::os::raw::c_long;
use std::ptr;

use app_units::Au;
use freetype_sys::{
    FT_F26Dot6, FT_FACE_FLAG_COLOR, FT_FACE_FLAG_FIXED_SIZES, FT_FACE_FLAG_MULTIPLE_MASTERS,
    FT_FACE_FLAG_SCALABLE, FT_Face, FT_Get_Sfnt_Table, FT_Int32, FT_LOAD_COLOR, FT_LOAD_DEFAULT,
    FT_Long, FT_Pos, FT_Select_Size, FT_Set_Char_Size, FT_Short, FT_UInt64, FT_ULong, FT_UShort,
    TT_OS2, ft_sfnt_os2,
};

use crate::font::FontTableTag;
use crate::platform::font::FontTable;
use crate::platform::freetype::FT_Load_Sfnt_Table;
use crate::platform::freetype::font::FT_LOAD_TARGET_LIGHT;

/// Data from the OS/2 table of an OpenType font.
/// See <https://www.microsoft.com/typography/otspec/os2.htm>
#[derive(Debug)]
pub struct OS2Table {
    pub version: FT_UShort,
    pub x_average_char_width: FT_Short,
    pub us_weight_class: FT_UShort,
    pub us_width_class: FT_UShort,
    pub y_strikeout_size: FT_Short,
    pub y_strikeout_position: FT_Short,
    // According to specs OS/2 unicode ranges should be FT_UInt32.
    // <https://learn.microsoft.com/en-us/typography/opentype/spec/os2>
    // <https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6OS2.html>
    // However freetype choses FT_UInt64. Understand why?
    // <https://freetype.org/freetype2/docs/reference/ft2-truetype_tables.html#tt_os2>
    pub ul_unicode_range1: FT_UInt64,
    pub ul_unicode_range2: FT_UInt64,
    pub ul_unicode_range3: FT_UInt64,
    pub ul_unicode_range4: FT_UInt64,
    pub sx_height: FT_Short,
}

pub trait FreeTypeFaceHelpers {
    fn scalable(self) -> bool;
    fn color(self) -> bool;
    fn has_axes(self) -> bool;
    fn set_size(self, pt_size: Au) -> Result<Au, &'static str>;
    fn glyph_load_flags(self) -> FT_Int32;
    fn os2_table(self) -> Option<OS2Table>;
    fn table_for_tag(self, tag: FontTableTag) -> Option<FontTable>;
}

impl FreeTypeFaceHelpers for FT_Face {
    fn scalable(self) -> bool {
        unsafe { (*self).face_flags & FT_FACE_FLAG_SCALABLE as c_long != 0 }
    }

    fn color(self) -> bool {
        unsafe { (*self).face_flags & FT_FACE_FLAG_COLOR as c_long != 0 }
    }

    fn has_axes(self) -> bool {
        unsafe { (*self).face_flags & FT_FACE_FLAG_MULTIPLE_MASTERS as c_long != 0 }
    }

    fn set_size(self, requested_size: Au) -> Result<Au, &'static str> {
        if self.scalable() {
            let size_in_fixed_point = (requested_size.to_f64_px() * 64.0 + 0.5) as FT_F26Dot6;
            let result = unsafe { FT_Set_Char_Size(self, size_in_fixed_point, 0, 72, 72) };
            if 0 != result {
                return Err("FT_Set_Char_Size failed");
            }
            return Ok(requested_size);
        }

        let requested_size = (requested_size.to_f64_px() * 64.0) as FT_Pos;
        let get_size_at_index = |index| unsafe {
            (
                (*(*self).available_sizes.offset(index as isize)).x_ppem,
                (*(*self).available_sizes.offset(index as isize)).y_ppem,
            )
        };

        let mut best_index = 0;
        let mut best_size = get_size_at_index(0);
        let mut best_dist = best_size.1 - requested_size;
        for strike_index in 1..unsafe { (*self).num_fixed_sizes } {
            let new_scale = get_size_at_index(strike_index);
            let new_distance = new_scale.1 - requested_size;

            // Distance is positive if strike is larger than desired size,
            // or negative if smaller. If previously a found smaller strike,
            // then prefer a larger strike. Otherwise, minimize distance.
            if (best_dist < 0 && new_distance >= best_dist) || new_distance.abs() <= best_dist {
                best_dist = new_distance;
                best_size = new_scale;
                best_index = strike_index;
            }
        }

        if 0 == unsafe { FT_Select_Size(self, best_index) } {
            Ok(Au::from_f64_px(best_size.1 as f64 / 64.0))
        } else {
            Err("FT_Select_Size failed")
        }
    }

    fn glyph_load_flags(self) -> FT_Int32 {
        let mut load_flags = FT_LOAD_DEFAULT;

        // Default to slight hinting, which is what most
        // Linux distros use by default, and is a better
        // default than no hinting.
        // TODO(gw): Make this configurable.
        load_flags |= FT_LOAD_TARGET_LIGHT as i32;

        let face_flags = unsafe { (*self).face_flags };
        if (face_flags & (FT_FACE_FLAG_FIXED_SIZES as FT_Long)) != 0 {
            // We only set FT_LOAD_COLOR if there are bitmap strikes; COLR (color-layer) fonts
            // will be handled internally in Servo. In that case WebRender will just be asked to
            // paint individual layers.
            load_flags |= FT_LOAD_COLOR;
        }

        load_flags as FT_Int32
    }

    fn os2_table(self) -> Option<OS2Table> {
        unsafe {
            let os2 = FT_Get_Sfnt_Table(self, ft_sfnt_os2) as *mut TT_OS2;
            let valid = !os2.is_null() && (*os2).version != 0xffff;

            if !valid {
                return None;
            }

            Some(OS2Table {
                version: (*os2).version,
                x_average_char_width: (*os2).xAvgCharWidth,
                us_weight_class: (*os2).usWeightClass,
                us_width_class: (*os2).usWidthClass,
                y_strikeout_size: (*os2).yStrikeoutSize,
                y_strikeout_position: (*os2).yStrikeoutPosition,
                ul_unicode_range1: (*os2).ulUnicodeRange1,
                ul_unicode_range2: (*os2).ulUnicodeRange2,
                ul_unicode_range3: (*os2).ulUnicodeRange3,
                ul_unicode_range4: (*os2).ulUnicodeRange4,
                sx_height: (*os2).sxHeight,
            })
        }
    }

    fn table_for_tag(self, tag: FontTableTag) -> Option<FontTable> {
        let tag = tag as FT_ULong;
        // Zero value in tag check must be added.
        // Check wether Freetype checks for invalid tag...

        unsafe {
            // Get the length of the table
            let mut len = 0;
            if 0 != FT_Load_Sfnt_Table(self, tag, 0, ptr::null_mut(), &mut len) {
                return None;
            }
            // Get the data
            let mut buf = vec![0_u8; len as usize];
            // let Ok(font_table) = FontTable::new_four_alligned(len.try_into().unwrap()) else {
            //     return None;
            // };

            if 0 != FT_Load_Sfnt_Table(self, tag, 0, buf.as_mut_ptr(), &mut len) {
                return None;
            }
            return Some(FontTable::new(buf));
        }
    }
}
