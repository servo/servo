/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr;

use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use freetype_sys::{
    FT_Err_Missing_Property, FT_Error, FT_Face, FT_Fixed, FT_Get_Sfnt_Name, FT_Int32, FT_MM_Var,
    FT_Set_Var_Design_Coordinates, FT_SfntName, FT_UInt, FT_ULong, FT_Var_Axis, FTErrorMethods,
};
use itertools::enumerate;
use log::log_enabled;
use parking_lot::ReentrantMutex;
use smallvec::SmallVec;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use webrender_api::FontInstanceFlags;

use crate::font::{FontTableTag, ot_tag_to_string};
use crate::platform::freetype::font::{FT_VARIATION_WEIGHT_TAG, FT_VARIATION_WIDTH_TAG, FontTable};
use crate::platform::freetype::freetype_errors::CustomFtErrorMethods;
use crate::platform::freetype::freetype_face_helpers::{FreeTypeFaceHelpers, OS2Table};
use crate::platform::freetype::freetype_variations_helpers::FreeTypeVariationsHelpers;
use crate::system_font_service::FontIdentifier;

pub trait FreeTypeComplexFaceHelpers {
    fn set_variations(self, variations: SmallVec<[(FT_ULong, FT_Fixed); 4]>) -> FT_Error;
}

// This is not working and trait aliases is still unstable
// pub trait PlatformFaceHelpers: FreeTypeFaceHelpers + FreeTypeVariationsHelpers + FreeTypeComplexHelpers {}
// impl<T: FreeTypeFaceHelpers + FreeTypeVariationsHelpers + FreeTypeComplexHelpers> PlatformFaceHelpers for T {}

#[derive(Debug)]
pub struct PlatformFontFaceHandle {
    pub face: FT_Face,
    pub variation_axes: *mut FT_MM_Var,
}

impl FreeTypeComplexFaceHelpers for &PlatformFontFaceHandle {
    fn set_variations(self, variations: SmallVec<[(FT_ULong, FT_Fixed); 4]>) -> FT_Error {
        assert!(!self.variation_axes.is_null());

        let Ok(mut parsed_axes) = self.variation_axes.get_variations() else {
            return FT_Err_Missing_Property;
        };
        let num_axis: FT_UInt = parsed_axes.len().try_into().unwrap();

        let init_ft_fixed: FT_Fixed = 0;
        let mut variation_coords: Vec<FT_Fixed> = vec![init_ft_fixed; num_axis as usize];
        let mut name_record = FT_SfntName {
            platform_id: 0,
            encoding_id: 0,
            name_id: 0,
            language_id: 0,
            string: ptr::null_mut(),
            string_len: 0,
        };

        let mut var_axis_name = String::new();
        for (index, axis_ptr) in enumerate(parsed_axes) {
            let axis: FT_Var_Axis = unsafe { *axis_ptr };
            let result: FT_Error = unsafe {
                FT_Get_Sfnt_Name(self.face, axis.strid, &mut name_record as *mut FT_SfntName)
            };
            if result.succeeded() {
                var_axis_name = unsafe {
                    String::from_utf8_unchecked(Vec::from_raw_parts(
                        name_record.string,
                        name_record.string_len.try_into().unwrap(),
                        name_record.string_len.try_into().unwrap(),
                    ))
                }
            } else {
                let error_string = result.ft_get_error_message();
                log::warn!(
                    "Freetype2. Unable to read variation axis name. FT_error: {}",
                    error_string
                );
            };
            if log_enabled!(log::Level::Debug) {
                log::warn!(
                    "Freetype2 var axis in font \
                    variation {:?} \
                    converted_min {:?} \
                    converted_value {:?} \
                    converted_max {:?} \
                    converted_tag {:?} \
                    name record: {:?} \
                    name record string: {:?}",
                    axis,
                    (axis.minimum / 65536) as f64,
                    (axis.def / 65536) as f64,
                    (axis.maximum / 65536) as f64,
                    ot_tag_to_string(axis.tag.try_into().unwrap()),
                    name_record,
                    var_axis_name
                );
            }
            // First setup default value
            variation_coords[index] = axis.def;
            // Then if we have value requested by user, setup this value!
            if FT_VARIATION_WEIGHT_TAG == axis.tag {
                let entry = variations
                    .iter()
                    .find(|(axis_tag, _)| FT_VARIATION_WEIGHT_TAG == *axis_tag);
                if let Some((_, axis_value)) = entry {
                    log::warn!(
                        "Freetype2. Found weight variation axis! setup weight: {}",
                        axis_value
                    );
                    if axis.minimum <= *axis_value && *axis_value <= axis.maximum {
                        variation_coords[index] = *axis_value;
                    }
                }
            }
            if FT_VARIATION_WIDTH_TAG == axis.tag {
                let entry = variations
                    .iter()
                    .find(|(axis_tag, _)| FT_VARIATION_WIDTH_TAG == *axis_tag);
                if let Some((_, axis_value)) = entry {
                    log::warn!(
                        "Freetype2. Found width variation axis! setup width: {}",
                        axis_value
                    );
                    if axis.minimum <= *axis_value && *axis_value <= axis.maximum {
                        variation_coords[index] = *axis_value;
                    }
                }
            }
        }

        // General way to setup and check axis information. Will work with all fonts
        unsafe {
            FT_Set_Var_Design_Coordinates(
                self.face,
                num_axis,
                variation_coords.as_ptr() as *const FT_Fixed,
            )
        }
    }
}

impl FreeTypeFaceHelpers for &PlatformFontFaceHandle {
    fn scalable(self) -> bool {
        self.face.scalable()
    }

    fn color(self) -> bool {
        self.face.color()
    }

    fn glyph_load_flags(self) -> FT_Int32 {
        self.face.glyph_load_flags()
    }

    fn has_axes(self) -> bool {
        self.face.has_axes()
    }

    fn os2_table(self) -> Option<OS2Table> {
        self.face.os2_table()
    }

    fn set_size(self, pt_size: Au) -> Result<Au, &'static str> {
        self.face.set_size(pt_size)
    }

    fn table_for_tag(self, tag: FontTableTag) -> Option<FontTable> {
        self.face.table_for_tag(tag)
    }
}

impl FreeTypeVariationsHelpers for &PlatformFontFaceHandle {
    fn get_variations(self) -> Result<Vec<*mut FT_Var_Axis>, FT_Error> {
        self.variation_axes.get_variations()
    }
}
