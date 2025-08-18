/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{CStr, c_long};
use std::ptr;

use app_units::Au;
use freetype_sys::{
    FT_Done_Face, FT_Done_MM_Var, FT_F26Dot6, FT_FACE_FLAG_COLOR, FT_FACE_FLAG_FIXED_SIZES,
    FT_FACE_FLAG_SCALABLE, FT_Face, FT_FaceRec, FT_Fixed, FT_Get_MM_Var, FT_HAS_MULTIPLE_MASTERS,
    FT_Int32, FT_LOAD_COLOR, FT_LOAD_DEFAULT, FT_Long, FT_MM_Var, FT_New_Face, FT_New_Memory_Face,
    FT_Pos, FT_Select_Size, FT_Set_Char_Size, FT_Set_Var_Design_Coordinates, FT_UInt,
    FTErrorMethods,
};
use webrender_api::FontVariation;

use crate::platform::freetype::library_handle::FreeTypeLibraryHandle;

// This constant is not present in the freetype
// bindings due to bindgen not handling the way
// the macro is defined.
const FT_LOAD_TARGET_LIGHT: FT_UInt = 1 << 16;

/// A safe wrapper around [FT_Face].
#[derive(Debug)]
pub(crate) struct FreeTypeFace {
    /// ## Safety Invariant
    /// The pointer must have been returned from [FT_New_Face] or [FT_New_Memory_Face]
    /// and must not be freed before `FreetypeFace::drop` is called.
    face: ptr::NonNull<FT_FaceRec>,
}

impl FreeTypeFace {
    pub(crate) fn new_from_memory(
        library: &FreeTypeLibraryHandle,
        data: &[u8],
    ) -> Result<Self, &'static str> {
        let mut face = ptr::null_mut();
        let result = unsafe {
            FT_New_Memory_Face(
                library.freetype_library,
                data.as_ptr(),
                data.len() as FT_Long,
                0,
                &mut face,
            )
        };

        if 0 != result {
            return Err("Could not create FreeType face");
        }
        let Some(face) = ptr::NonNull::new(face) else {
            return Err("Could not create FreeType face");
        };

        Ok(Self { face })
    }

    pub(crate) fn new_from_file(
        library: &FreeTypeLibraryHandle,
        filename: &CStr,
        index: u32,
    ) -> Result<Self, &'static str> {
        let mut face = ptr::null_mut();
        let result = unsafe {
            FT_New_Face(
                library.freetype_library,
                filename.as_ptr(),
                index as FT_Long,
                &mut face,
            )
        };

        if 0 != result {
            return Err("Could not create FreeType face");
        }
        let Some(face) = ptr::NonNull::new(face) else {
            return Err("Could not create FreeType face");
        };

        Ok(Self { face })
    }

    pub(crate) fn as_ref(&self) -> &FT_FaceRec {
        unsafe { self.face.as_ref() }
    }

    pub(crate) fn as_ptr(&self) -> FT_Face {
        self.face.as_ptr()
    }

    /// Return true iff the font face flags contain [FT_FACE_FLAG_SCALABLE].
    pub(crate) fn scalable(&self) -> bool {
        self.as_ref().face_flags & FT_FACE_FLAG_SCALABLE as c_long != 0
    }

    /// Return true iff the font face flags contain [FT_FACE_FLAG_COLOR].
    pub(crate) fn color(&self) -> bool {
        self.as_ref().face_flags & FT_FACE_FLAG_COLOR as c_long != 0
    }

    /// Scale the font to the given size if it is scalable, or select the closest
    /// available size if it is not, preferring larger sizes over smaller ones.
    ///
    /// Returns the selected size on success and a error message on failure
    pub(crate) fn set_size(&self, requested_size: Au) -> Result<Au, &'static str> {
        if self.scalable() {
            let size_in_fixed_point = (requested_size.to_f64_px() * 64.0 + 0.5) as FT_F26Dot6;
            let result =
                unsafe { FT_Set_Char_Size(self.face.as_ptr(), size_in_fixed_point, 0, 72, 72) };
            if 0 != result {
                return Err("FT_Set_Char_Size failed");
            }
            return Ok(requested_size);
        }

        let requested_size = (requested_size.to_f64_px() * 64.0) as FT_Pos;
        let get_size_at_index = |index| unsafe {
            (
                (*self.as_ref().available_sizes.offset(index as isize)).x_ppem,
                (*self.as_ref().available_sizes.offset(index as isize)).y_ppem,
            )
        };

        let mut best_index = 0;
        let mut best_size = get_size_at_index(0);
        let mut best_dist = best_size.1 - requested_size;
        for strike_index in 1..self.as_ref().num_fixed_sizes {
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

        if 0 == unsafe { FT_Select_Size(self.face.as_ptr(), best_index) } {
            Ok(Au::from_f64_px(best_size.1 as f64 / 64.0))
        } else {
            Err("FT_Select_Size failed")
        }
    }

    /// Select a reasonable set of glyph loading flags for the font.
    pub(crate) fn glyph_load_flags(&self) -> FT_Int32 {
        let mut load_flags = FT_LOAD_DEFAULT;

        // Default to slight hinting, which is what most
        // Linux distros use by default, and is a better
        // default than no hinting.
        // TODO(gw): Make this configurable.
        load_flags |= FT_LOAD_TARGET_LIGHT as i32;

        let face_flags = self.as_ref().face_flags;
        if (face_flags & (FT_FACE_FLAG_FIXED_SIZES as FT_Long)) != 0 {
            // We only set FT_LOAD_COLOR if there are bitmap strikes; COLR (color-layer) fonts
            // will be handled internally in Servo. In that case WebRender will just be asked to
            // paint individual layers.
            load_flags |= FT_LOAD_COLOR;
        }

        load_flags as FT_Int32
    }

    /// Applies to provided variations to the font face.
    ///
    /// Returns the normalized font variations, which are clamped
    /// to fit within the range of their respective axis. Variation
    /// values for nonexistent axes are not included.
    pub(crate) fn set_variations_for_font(
        &self,
        variations: &[FontVariation],
        library: &FreeTypeLibraryHandle,
    ) -> Result<Vec<FontVariation>, &'static str> {
        if !FT_HAS_MULTIPLE_MASTERS(self.as_ptr()) ||
            variations.is_empty() ||
            !servo_config::pref!(layout_variable_fonts_enabled)
        {
            // Nothing to do
            return Ok(vec![]);
        }

        // Query variation axis of font
        let mut mm_var: *mut FT_MM_Var = ptr::null_mut();
        let result = unsafe { FT_Get_MM_Var(self.as_ptr(), &mut mm_var as *mut _) };
        if !result.succeeded() {
            return Err("Failed to query font variations");
        }

        // Prepare values for each axis. These are either the provided values (if any) or the default
        // ones for the axis.
        let num_axis = unsafe { (*mm_var).num_axis } as usize;
        let mut normalized_axis_values = Vec::with_capacity(variations.len());
        let mut coords = vec![0; num_axis];
        for (index, coord) in coords.iter_mut().enumerate() {
            let axis_data = unsafe { &*(*mm_var).axis.add(index) };
            let Some(variation) = variations
                .iter()
                .find(|variation| variation.tag == axis_data.tag as u32)
            else {
                *coord = axis_data.def;
                continue;
            };

            // Freetype uses a 16.16 fixed point format for variation values
            let shift_factor = 16.0_f32.exp2();
            let min_value = axis_data.minimum as f32 / shift_factor;
            let max_value = axis_data.maximum as f32 / shift_factor;
            normalized_axis_values.push(FontVariation {
                tag: variation.tag,
                value: variation.value.min(max_value).max(min_value),
            });

            *coord = (variation.value * shift_factor) as FT_Fixed;
        }

        // Free the MM_Var structure
        unsafe {
            FT_Done_MM_Var(library.freetype_library, mm_var);
        }

        // Set the values for each variation axis
        let result = unsafe {
            FT_Set_Var_Design_Coordinates(self.as_ptr(), coords.len() as u32, coords.as_ptr())
        };
        if !result.succeeded() {
            return Err("Could not set variations for font face");
        }

        Ok(normalized_axis_values)
    }
}

/// FT_Face can be used in multiple threads, but from only one thread at a time.
/// See <https://freetype.org/freetype2/docs/reference/ft2-face_creation.html#ft_face>.
unsafe impl Send for FreeTypeFace {}

impl Drop for FreeTypeFace {
    fn drop(&mut self) {
        // The FreeType documentation says that both `FT_New_Face` and `FT_Done_Face`
        // should be protected by a mutex.
        // See https://freetype.org/freetype2/docs/reference/ft2-library_setup.html.
        let _guard = FreeTypeLibraryHandle::get().lock();
        if unsafe { FT_Done_Face(self.face.as_ptr()) } != 0 {
            panic!("FT_Done_Face failed");
        }
    }
}
