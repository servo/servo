/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use freetype_sys::{FT_Err_Missing_Property, FT_Error, FT_MM_Var, FT_Var_Axis};

use crate::platform::freetype::font::{FT_VARIATION_WEIGHT_TAG, FT_VARIATION_WIDTH_TAG};

pub trait FreeTypeVariationsHelpers {
    fn get_variations(self) -> Result<Vec<*mut FT_Var_Axis>, FT_Error>;
}

pub trait FreeTypeSingleVariationAxisHelpers {
    unsafe fn is_width(self) -> bool;
    unsafe fn is_weight(self) -> bool;
}

impl FreeTypeVariationsHelpers for *mut FT_MM_Var {
    fn get_variations(self) -> Result<Vec<*mut FT_Var_Axis>, FT_Error> {
        let num_axis: u32 = unsafe { (*self).num_axis };
        if num_axis == 0 {
            log::warn!("Freetype 2. Variation axis not found!");
            return Err(FT_Err_Missing_Property);
        }
        log::debug!(
            "Freetype2 \
            variation axes count: {:?}",
            num_axis
        );

        let mut parsed_axes = Vec::<*mut FT_Var_Axis>::new();
        for i in 0..num_axis as usize {
            let elem_ptr: *mut FT_Var_Axis = unsafe { (*self).axis.wrapping_add(i) };
            if elem_ptr.is_null() {
                log::error!("Freetype 2. Unable to get variation axis ptr");
                continue;
            }
            parsed_axes.push(elem_ptr);
        }
        Ok(parsed_axes)
    }
}

impl FreeTypeSingleVariationAxisHelpers for *mut FT_Var_Axis {
    unsafe fn is_width(self) -> bool {
        unsafe { (*self).tag == FT_VARIATION_WIDTH_TAG }
    }
    unsafe fn is_weight(self) -> bool {
        unsafe { (*self).tag == FT_VARIATION_WEIGHT_TAG }
    }
}
