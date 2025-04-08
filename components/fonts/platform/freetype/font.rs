/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::error::Error;
use std::ffi::{CString, NulError};
use std::ops::RangeInclusive;
use std::os::raw::c_long;
use std::{mem, ptr};

use app_units::Au;
use euclid::default::{Point2D, Rect, Size2D};
use freetype_sys::{
    FT_Done_Face, FT_Done_MM_Var, FT_Error, FT_F26Dot6, FT_Face, FT_Fixed, FT_Get_Char_Index,
    FT_Get_Kerning, FT_Get_MM_Var, FT_Get_Sfnt_Table, FT_GlyphSlot, FT_KERNING_DEFAULT,
    FT_LOAD_DEFAULT, FT_LOAD_NO_HINTING, FT_Load_Glyph, FT_Long, FT_MM_Var, FT_MulFix, FT_New_Face,
    FT_New_Memory_Face, FT_STYLE_FLAG_ITALIC, FT_Short, FT_Size_Metrics, FT_SizeRec, FT_UInt,
    FT_ULong, FT_UShort, FT_Vector, FTErrorMethods, ft_sfnt_head, FT_Open_Face, FT_Open_Args,
    FT_OPEN_PATHNAME, FT_Int64
};
use icu_locid::LanguageIdentifier;
use icu_locid::subtags::Language;
use log::debug;
use parking_lot::ReentrantMutex;
use smallvec::SmallVec;
use style::Zero;
use style::values::computed::{
    FontStretch as StyleFontStretch, FontStyle as StyleFontStyle, FontWeight as StyleFontWeight,
};
use unicode_language::Match as LanguageMatch;
use webrender_api::FontInstanceFlags;

use super::LocalFontIdentifier;
use super::library_handle::FreeTypeLibraryHandle;
use crate::font::{
    FontMetrics, FontTableMethods, FontTableTag, FractionalPixel, PlatformFontMethods,
    ot_tag_to_string,
};
use crate::font_template::FontTemplateDescriptor;
use crate::glyph::GlyphId;
use crate::platform::freetype::face::{FreeTypeComplexFaceHelpers, PlatformFontFaceHandle};
use crate::platform::freetype::freetype_errors::CustomFtErrorMethods;
use crate::platform::freetype::freetype_face_helpers::FreeTypeFaceHelpers;
use crate::platform::freetype::freetype_truetype_unicode_ranges::convert_unicode_ranges;
use crate::platform::freetype::freetype_variations_helpers::{
    FreeTypeSingleVariationAxisHelpers, FreeTypeVariationsHelpers,
};
use crate::system_font_service::FontIdentifier;
use crate::{FontData, ot_tag};

// Special constant to convert freetype 16.16 fixed point Type
pub const FT_FIXED_CONVERSION_CONSTANT: f64 = (1 << 16) as f64;
// Special constant to convert freetype 26.6 fixed point Type
pub const FT_F26DOT6_CONVERSION_CONSTANT: f64 = (1 << 6) as f64;

// This constant is not present in the freetype
// bindings due to bindgen not handling the way
// the macro is defined.
pub const FT_LOAD_TARGET_LIGHT: FT_UInt = 1 << 16;

/// For any negative value of face_index, face->num_faces gives the number of faces within the font file.
/// For the negative value ‘-(N+1)’ (with ‘N’ a non-negative 16-bit value), bits 16-30 in face->style_flags
/// give the number of named instances in face ‘N’ if we have a variation font (or zero otherwise).
pub const FT_OPEN_FACE_SEARCH_CONSTANT: FT_Long = -(0 + 1);

pub const FT_VARIATION_WEIGHT_TAG: FT_ULong = ot_tag!('w', 'g', 'h', 't') as u64;
pub const FT_VARIATION_WIDTH_TAG: FT_ULong = ot_tag!('w', 'd', 't', 'h') as u64;

/// Convert FreeType-style 26.6 fixed point to an [`f64`].
fn fixed_26_dot_6_to_float(fixed: FT_F26Dot6) -> f64 {
    fixed as f64 / FT_F26DOT6_CONVERSION_CONSTANT
}

/// Convert FreeType-style 16.16 fixed point to an [`f64`].
fn fixed_16_dot_16_to_float(fixed: FT_Fixed) -> f64 {
    fixed as f64 / FT_FIXED_CONVERSION_CONSTANT
}

#[derive(Debug)]
pub struct FontTable {
    buffer: Vec<u8>,
}

impl FontTable {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }
}

impl FontTableMethods for FontTable {
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct PlatformFont {
    face_handle: ReentrantMutex<PlatformFontFaceHandle>,
    requested_face_size: Au,
    actual_face_size: Au,
}

impl PlatformFont {
    /// Find the scale to use for metrics of unscalable fonts. Unscalable fonts, those using bitmap
    /// glyphs, are scaled after glyph rasterization. In order for metrics to match the final scaled
    /// font, we need to scale them based on the final size and the actual font size.
    fn unscalable_font_metrics_scale(&self) -> f64 {
        self.requested_face_size.to_f64_px() / self.actual_face_size.to_f64_px()
    }

    fn ft_weight_from_css(css_weight: &StyleFontWeight) -> FT_Fixed {
        ((css_weight.value() as u32) * 65536) as FT_Fixed
    }

    fn ft_width_from_css(css_stretch: StyleFontStretch) -> FT_Fixed {
        let stretch_val: f32 = match css_stretch {
            StyleFontStretch::ULTRA_CONDENSED => 50.0,
            StyleFontStretch::EXTRA_CONDENSED => 62.0,
            StyleFontStretch::CONDENSED => 75.0,
            StyleFontStretch::SEMI_CONDENSED => 87.0,
            StyleFontStretch::NORMAL => 100.0,
            StyleFontStretch::SEMI_EXPANDED => 112.0,
            StyleFontStretch::EXPANDED => 125.0,
            StyleFontStretch::EXTRA_EXPANDED => 150.0,
            StyleFontStretch::ULTRA_EXPANDED => 200.0,
            _ => {
                log::warn!("Unsupported stretch value!");
                100.0
            },
        };
        ((stretch_val as u32) * 65536) as FT_Fixed
    }

    fn get_number_of_named_faces_in_font(
        library: &FreeTypeLibraryHandle,
        font_identifier: &LocalFontIdentifier
    ) -> Result<(FT_Int64, FT_Int64), &'static str> {
        let mut pathname_raw_bytes_vec = Vec::<u8>::from(&*font_identifier.path);
        let mut open_args = FT_Open_Args {
            flags: FT_OPEN_PATHNAME,
            // Memory base is not used because of FT_OPEN_PATHNAME flag
            memory_base: ptr::null_mut(),
            memory_size: 0,
            // pathname is used
            pathname: pathname_raw_bytes_vec.as_mut_slice().as_mut_ptr(),
            // stream is not used;
            stream: ptr::null_mut(),
            // not specified, Freetype will try all drivers in the list
            // later try to explicitely set OT/TT driver to save time
            driver: ptr::null_mut(),
            num_params: 0,
            params: ptr::null_mut(),
        };
        // Service face object is not the face that we will use!
        // We get it to gain information about faces within font file with Freetype2 library
        // Later we will search face that conforms with our parameters within all faces
        let mut service_face_object: FT_Face = ptr::null_mut();
        // After this call service_face_object should be populated with
        // information about first `font face` named variation indexes in the font file
        let result = unsafe {
            FT_Open_Face(
                library.freetype_library,
                &open_args as *const FT_Open_Args,
                FT_OPEN_FACE_SEARCH_CONSTANT,
                &mut service_face_object,
            )
        };
        if !result.succeeded() || service_face_object.is_null() {
            // We want to have more logs on OpenHarmony,
            // Linux and Android must not be affected;
            #[cfg(any(target_env = "ohos", ohos_mock))]
            {
                log::error!(
                    "Freetype2. Unsupported font format in provided file! FT_error: {}",
                    result.ft_get_error_message()
                );
            }
            return Err("Freetype2. Unsupported font format in provided file!");
        }

        let num_faces_in_font_file = unsafe {(*service_face_object).num_faces };
        // Bits 16-30 hold the number of named instances available for the current face
        // if we have a GX or OpenType variation (sub)font. Bit 31 is always zero (that is,
        // style_flags is always a positive value).
        // Note that a variation font has always at least one named instance, namely the default instance.
        let num_named_instances_in_default_face = unsafe {(*service_face_object).style_flags} >> 16;
        let result = unsafe { FT_Done_Face(service_face_object) };
        if !result.succeeded() {
            // We want to have more logs on OpenHarmony,
            // Linux and Android must not be affected;
            #[cfg(any(target_env = "ohos", ohos_mock))]
            {
                log::error!(
                    "Freetype 2. Unable to destroy font face object. Possible memory leak! FT_error: {}",
                    result.ft_get_error_message()
                );
            }
            // Should we panic here?
            return Err("Unable to destroy font face object. Possible memory leak!");
        }

        log::debug!(
            "Freetype 2. \
            Number of faces in font file: {} \
            Number of named instances in default face: {}",
            num_faces_in_font_file,
            num_named_instances_in_default_face
        );

        return Ok((num_faces_in_font_file, num_named_instances_in_default_face))
    }

    // fn search_within_named_instances() {
    //     let (num_faces_in_font_file, num_of_named_instances_in_first_face) =
    //     match Self::get_number_of_named_faces_in_font(&library, &font_identifier) {
    //         Err(e) => return Err(e),
    //         Ok((num_faces_in_font_file, num_of_named_instances_in_first_face)) =>
    //             (num_faces_in_font_file, num_of_named_instances_in_first_face),
    //     };

    //     let mut pathname_raw_bytes_vec = Vec::<u8>::from(&*font_identifier.path);
    //     // FT_PARAM_TAG_INCREMENTAL is the only parameter that currently matters...
    //     // So FT_OPEN_PARAMS will not help us to find correct font faster
    //     let mut open_args = FT_Open_Args {
    //         flags: FT_OPEN_PATHNAME,
    //         // Memory base is not used because of FT_OPEN_PATHNAME flag
    //         memory_base: ptr::null_mut(),
    //         memory_size: 0,
    //         // pathname is used
    //         pathname: pathname_raw_bytes_vec.as_mut_slice().as_mut_ptr(),
    //         // stream is not used;
    //         stream: ptr::null_mut(),
    //         // not specified, Freetype will try all drivers in the list
    //         // later try to explicitely set OT/TT driver to save time
    //         driver: ptr::null_mut(),
    //         // params is not used;
    //         num_params: 0,
    //         params: ptr::null_mut(),
    //     };

    //     let mut number_of_named_instances = num_of_named_instances_in_first_face;
    //     for face_counter in 0..num_faces_in_font_file {
    //         for named_instance_counter in 0..number_of_named_instances {
    //             let mut service_face_object: FT_Face = ptr::null_mut();
    //             // After this call service_face_object should be populated with
    //             // information about first `font face` named variation indexes in the font file
    //             let face_idx = face_counter + (named_instance_counter << 16);
    //             let result = unsafe {
    //                 FT_Open_Face(
    //                     library.freetype_library,
    //                     &open_args as *const FT_Open_Args,
    //                     face_idx,
    //                     &mut service_face_object,
    //                 )
    //             };
    //             if(!service_face_object.has_axes()) {
    //                 let result = unsafe { FT_Done_Face(service_face_object) };
    //                 if !result.succeeded() {
    //                     // We want to have more logs on OpenHarmony,
    //                     // Linux and Android must not be affected;
    //                     #[cfg(any(target_env = "ohos", ohos_mock))]
    //                     {
    //                         log::error!(
    //                             "Freetype2. We was unable to destroy font face object. Possible memory leak! FT_error: {}",
    //                             result.ft_get_error_message()
    //                         );
    //                     }
    //                     // Should we panic here?
    //                     return Err("We was unable to destroy font face object. Possible memory leak!");
    //                 };
    //                 continue
    //             };
    //             let mut variation_axes: *mut FT_MM_Var = ptr::null_mut();
    //             let result: FT_Error = unsafe { FT_Get_MM_Var(service_face_object, &mut variation_axes) };
    //             if !result.succeeded() || variation_axes.is_null() {
    //                 let error_string = result.ft_get_error_message();
    //                 log::error!(
    //                     "Freetype2. We was not able to setup variation axis. FT_error: {}",
    //                     error_string
    //                 );
    //             }
    //             let axis_vector = variation_axes.get_variations();
    //             // We can search variations here! But that will require architecture changes...
    //             // Add read-fonts crate from fontations to OpenHarmony FontList
    //             let result = unsafe { FT_Done_Face(service_face_object) };
    //             if !result.succeeded() {
    //                 // We want to have more logs on OpenHarmony,
    //                 // Linux and Android must not be affected;
    //                 #[cfg(any(target_env = "ohos", ohos_mock))]
    //                 {
    //                     log::error!(
    //                         "Freetype2. We was unable to destroy font face object. Possible memory leak! FT_error: {}",
    //                         result.ft_get_error_message()
    //                     );
    //                 }
    //                 // Should we panic here?
    //                 return Err("We was unable to destroy font face object. Possible memory leak!");
    //             }
    //         }
    //     }

    //     log::warn!(
    //         "Freetype2. \
    //         Number of faces in font file: {} \
    //         Number of named instances in default face: {}",
    //         num_faces_in_font_file,
    //         num_of_named_instances_in_first_face
    //     );
    // }
}

// FT_Face can be used in multiple threads, but from only one thread at a time.
// It's protected with a ReentrantMutex for PlatformFont.
// See https://freetype.org/freetype2/docs/reference/ft2-face_creation.html#ft_face.
unsafe impl Sync for PlatformFont {}
unsafe impl Send for PlatformFont {}

impl Drop for PlatformFont {
    fn drop(&mut self) {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        let var_axes = face_handle.variation_axes;
        assert!(!face.is_null());
        unsafe {
            // The FreeType documentation says that both `FT_New_Face` and `FT_Done_Face`
            // should be protected by a mutex.
            // See https://freetype.org/freetype2/docs/reference/ft2-library_setup.html.
            let ft_library_handle = FreeTypeLibraryHandle::get().lock();
            if !var_axes.is_null() {
                if FT_Done_MM_Var(ft_library_handle.freetype_library, var_axes) != 0 {
                    panic!("FT_Done_MM_Var failed");
                }
            }
            if FT_Done_Face(face) != 0 {
                panic!("FT_Done_Face failed");
            }
        }
    }
}

impl PlatformFontMethods for PlatformFont {
    fn new_from_data(
        font_identifier: FontIdentifier,
        font_data: &FontData,
        requested_size: Option<Au>,
        requested_weight: Option<StyleFontWeight>,
        requested_stretch: Option<StyleFontStretch>,
    ) -> Result<PlatformFont, &'static str> {
        let library = FreeTypeLibraryHandle::get().lock();
        let data: &[u8] = font_data.as_ref();
        let mut face: FT_Face = ptr::null_mut();
        let result = unsafe {
            FT_New_Memory_Face(
                library.freetype_library,
                data.as_ptr(),
                data.len() as FT_Long,
                0, /* face_index */
                &mut face,
            )
        };

        if 0 != result || face.is_null() {
            // We want to have more logs on OpenHarmony,
            // Linux and Android must not be affected;
            #[cfg(any(target_env = "ohos", ohos_mock))]
            {
                let ft_error_code: FT_Error = result as FT_Error;
                let error_string = ft_error_code.ft_get_error_message();
                log::error!("Could not create FreeType face. FT_error: {}", error_string);
            }
            return Err("Could not create FreeType face");
        }

        let mut variation_axes: *mut FT_MM_Var = ptr::null_mut();
        if face.has_axes() {
            let result: FT_Error = unsafe { FT_Get_MM_Var(face, &mut variation_axes) };
            if !result.succeeded() || variation_axes.is_null() {
                let error_string = result.ft_get_error_message();
                log::error!(
                    "Freetype2. Unable to setup variation axis. FT_error: {}",
                    error_string
                );
            }
        }

        // Create default font face;
        let face_handle = PlatformFontFaceHandle {
            face,
            variation_axes,
        };

        // From here we apply CSS styles provided by user;
        let (requested_face_size, actual_face_size) = match requested_size {
            Some(requested_size) => (requested_size, face_handle.set_size(requested_size)?),
            None => (Au::zero(), Au::zero()),
        };

        // Block bellow will prepare all parameters that will allow us to search font face with
        // style requested by user within font file
        // Size of 4 is chosen cause it is more propbable amount of variation axis within the font
        // <https://learn.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg>
        let mut variations = SmallVec::<[(FT_ULong, FT_Fixed); 4]>::new();
        if let Some(ref css_weight) = requested_weight {
            let ft_weight: FT_Fixed = Self::ft_weight_from_css(css_weight);
            variations.push((FT_VARIATION_WEIGHT_TAG, ft_weight));
        }

        if let Some(css_stretch) = requested_stretch {
            let ft_stretch = Self::ft_width_from_css(css_stretch);
            variations.push((FT_VARIATION_WIDTH_TAG, ft_stretch));
        }

        if face_handle.has_axes() {
            log::debug!("Freetype2. Attempt to set font variations.");
            let result: FT_Error = face_handle.set_variations(variations);
            if !result.succeeded() {
                log::error!(
                    "Freetype2. Error on face variation axis setup. FT_error: {:?} \
                    Program will not be interrupted, but face will come with default variations",
                    result.ft_get_error_message()
                );
            }
        }

        Ok(PlatformFont {
            face_handle: ReentrantMutex::new(face_handle),
            requested_face_size,
            actual_face_size,
        })
    }

    fn new_from_local_font_identifier(
        font_identifier: LocalFontIdentifier,
        requested_size: Option<Au>,
        requested_weight: Option<StyleFontWeight>,
        requested_stretch: Option<StyleFontStretch>,
    ) -> Result<PlatformFont, &'static str> {
        let library = FreeTypeLibraryHandle::get().lock();
        let filename = match CString::new(&*font_identifier.path) {
            Err(e) => {
                let e: NulError = e;
                // We want to have more logs on OpenHarmony,
                // Linux and Android must not be affected;
                #[cfg(any(target_env = "ohos", ohos_mock))]
                {
                    log::error!("{}\nCaused by: {}", e, e.source().unwrap());
                }
                return Err("filename contains NUL byte!");
            },
            Ok(data) => data,
        };
        let mut face: FT_Face = ptr::null_mut();
        // First we must create the face cause we can not pass nullptr to FT_Open_Face
        let result: FT_Error = unsafe {
            FT_New_Face(
                library.freetype_library,
                filename.as_ptr(),
                font_identifier.index() as FT_Long,
                &mut face,
            )
        };

        if !result.succeeded() || face.is_null() {
            // We want to have more logs on OpenHarmony,
            // Linux and Android must not be affected;
            #[cfg(any(target_env = "ohos", ohos_mock))]
            {
                log::error!(
                    "FreeType2. Could not create face. FT_error: {}",
                    result.ft_get_error_message()
                );
            }
            return Err("Could not create FreeType face");
        }

        let mut variation_axes: *mut FT_MM_Var = ptr::null_mut();
        if face.has_axes() {
            let result: FT_Error = unsafe { FT_Get_MM_Var(face, &mut variation_axes) };
            if !result.succeeded() || variation_axes.is_null() {
                let error_string = result.ft_get_error_message();
                log::error!(
                    "FreeType2. Unable to setup variation axis. FT_error: {}",
                    error_string
                );
            }
        }

        // Create default font face;
        let face_handle = PlatformFontFaceHandle {
            face: face,
            variation_axes,
        };

        // From here we apply CSS styles provided by user;
        let (requested_face_size, actual_face_size) = match requested_size {
            Some(requested_size) => (requested_size, face_handle.set_size(requested_size)?),
            None => (Au::zero(), Au::zero()),
        };

        // Block bellow will prepare all parameters that will allow us to search font face with
        // style requested by user within font file
        let mut variations = SmallVec::<[(FT_ULong, FT_Fixed); 4]>::new();
        if let Some(ref css_weight) = requested_weight {
            let ft_weight: FT_Fixed = Self::ft_weight_from_css(css_weight);
            variations.push((FT_VARIATION_WEIGHT_TAG, ft_weight));
        }

        if let Some(css_stretch) = requested_stretch {
            let ft_stretch = Self::ft_width_from_css(css_stretch);
            variations.push((FT_VARIATION_WIDTH_TAG, ft_stretch));
        }

        if face_handle.has_axes() {

            log::debug!("Freetype2. Trying to set font variations.");
            let result: FT_Error = face_handle.set_variations(variations);
            if !result.succeeded() {
                log::error!(
                    "Freetype2. Error on face variation axis setup. FT_error: {:?} \
                    Program will not be interrupted, but face will come with default variations",
                    result.ft_get_error_message()
                );
            }
        }

        Ok(PlatformFont {
            face_handle: ReentrantMutex::new(face_handle),
            requested_face_size,
            actual_face_size,
        })
    }

    fn descriptor(&self) -> FontTemplateDescriptor {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        let style = if unsafe { (*face).style_flags & FT_STYLE_FLAG_ITALIC as c_long != 0 } {
            StyleFontStyle::ITALIC
        } else {
            StyleFontStyle::NORMAL
        };

        let os2_table = face_handle.os2_table();
        let weight = os2_table
            .as_ref()
            .map(|os2| {
                StyleFontWeight::from_float(os2.us_weight_class as f32)
            })
            .unwrap_or_else(StyleFontWeight::normal);
        let stretch = os2_table
            .as_ref()
            .map(|os2| match os2.us_width_class {
                1 => StyleFontStretch::ULTRA_CONDENSED,
                2 => StyleFontStretch::EXTRA_CONDENSED,
                3 => StyleFontStretch::CONDENSED,
                4 => StyleFontStretch::SEMI_CONDENSED,
                5 => StyleFontStretch::NORMAL,
                6 => StyleFontStretch::SEMI_EXPANDED,
                7 => StyleFontStretch::EXPANDED,
                8 => StyleFontStretch::EXTRA_EXPANDED,
                9 => StyleFontStretch::ULTRA_EXPANDED,
                _ => StyleFontStretch::NORMAL,
            })
            .unwrap_or(StyleFontStretch::NORMAL);

        let unicode_ranges = os2_table
            .as_ref()
            .map(|os2| {
                Some(convert_unicode_ranges(
                    os2.ul_unicode_range1,
                    os2.ul_unicode_range2,
                    os2.ul_unicode_range3,
                    os2.ul_unicode_range4,
                ))
            })
            .unwrap_or(None);

        // Constant bellow decides the required overlap of ranges required by language
        // and ranges supported by font.
        // Check unicode_language crate
        // TODO(ddesyatkin): Choose correct constant here. Understand feasibility of using this
        // over parsing SFNT names. It seems that for the sake of Uniformity unicode_language crate is better
        const LANGUAGE_DETECTION_THRESHOLD: f64 = 0.8;
        let language_matches = unicode_language::detect(
            unicode_ranges
                .as_ref()
                .unwrap_or(&Vec::<RangeInclusive<u32>>::new())
                .iter()
                .map(|unicode_range| [unicode_range.start().clone(), unicode_range.end().clone()])
                .collect::<Vec<[u32; 2]>>(),
            LANGUAGE_DETECTION_THRESHOLD,
        );

        let languages: Vec<LanguageIdentifier> = language_matches
            .into_iter()
            .filter_map(|language_match: LanguageMatch| {
                Some(language_match.tag.parse::<Language>().ok()?)
            })
            .map(|language| LanguageIdentifier::from(language))
            .collect();

        FontTemplateDescriptor {
            weight: (weight, weight),
            stretch: (stretch, stretch),
            style: (style, style),
            languages: languages.into(),
            unicode_range: unicode_ranges,
        }
    }

    fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        unsafe {
            let idx = FT_Get_Char_Index(face, codepoint as FT_ULong);
            if idx != 0 as FT_UInt {
                Some(idx as GlyphId)
            } else {
                debug!(
                    "Invalid codepoint: U+{:04X} ('{}')",
                    codepoint as u32, codepoint
                );
                None
            }
        }
    }

    fn glyph_h_kerning(&self, first_glyph: GlyphId, second_glyph: GlyphId) -> FractionalPixel {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        let mut delta = FT_Vector { x: 0, y: 0 };
        unsafe {
            FT_Get_Kerning(
                face,
                first_glyph,
                second_glyph,
                FT_KERNING_DEFAULT,
                &mut delta,
            );
        }
        fixed_26_dot_6_to_float(delta.x) * self.unscalable_font_metrics_scale()
    }

    fn glyph_h_advance(&self, glyph: GlyphId) -> Option<FractionalPixel> {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        let load_flags = face_handle.glyph_load_flags();
        let result = unsafe { FT_Load_Glyph(face, glyph as FT_UInt, load_flags) };
        if 0 != result {
            debug!("Unable to load glyph {}. reason: {:?}", glyph, result);
            return None;
        }

        let void_glyph = unsafe { (*face).glyph };
        let slot: FT_GlyphSlot = void_glyph;
        assert!(!slot.is_null());

        let advance = unsafe { (*slot).metrics.horiAdvance };
        Some(fixed_26_dot_6_to_float(advance) * self.unscalable_font_metrics_scale())
    }

    fn metrics(&self) -> FontMetrics {
        let face_handle = self.face_handle.lock();
        let face_ptr = face_handle.face;
        let face = unsafe { &*face_ptr };

        // face.size is a *c_void in the bindings, presumably to avoid recursive structural types
        let freetype_size: &FT_SizeRec = unsafe { mem::transmute(&(*face.size)) };
        let freetype_metrics: &FT_Size_Metrics = &(freetype_size).metrics;

        let mut max_advance;
        let mut max_ascent;
        let mut max_descent;
        let mut line_height;
        let mut y_scale = 0.0;
        let mut em_height;
        if face_handle.scalable() {
            // Prefer FT_Size_Metrics::y_scale to y_ppem as y_ppem does not have subpixel accuracy.
            //
            // FT_Size_Metrics::y_scale is in 16.16 fixed point format.  Its (fractional) value is a
            // factor that converts vertical metrics from design units to units of 1/64 pixels, so
            // that the result may be interpreted as pixels in 26.6 fixed point format.
            //
            // This converts the value to a float without losing precision.
            y_scale = freetype_metrics.y_scale as f64 / 65535.0 / 64.0;

            max_advance = (face.max_advance_width as f64) * y_scale;
            max_ascent = (face.ascender as f64) * y_scale;
            max_descent = -(face.descender as f64) * y_scale;
            line_height = (face.height as f64) * y_scale;
            em_height = (face.units_per_EM as f64) * y_scale;
        } else {
            max_advance = fixed_26_dot_6_to_float(freetype_metrics.max_advance);
            max_ascent = fixed_26_dot_6_to_float(freetype_metrics.ascender);
            max_descent = -fixed_26_dot_6_to_float(freetype_metrics.descender);
            line_height = fixed_26_dot_6_to_float(freetype_metrics.height);

            em_height = freetype_metrics.y_ppem as f64;
            // FT_Face doc says units_per_EM and a bunch of following fields are "only relevant to
            // scalable outlines". If it's an sfnt, we can get units_per_EM from the 'head' table
            // instead; otherwise, we don't have a unitsPerEm value so we can't compute y_scale and
            // x_scale.
            let head = unsafe { FT_Get_Sfnt_Table(face_ptr, ft_sfnt_head) as *mut TtHeader };
            if !head.is_null() && unsafe { (*head).table_version != 0xffff } {
                // Bug 1267909 - Even if the font is not explicitly scalable, if the face has color
                // bitmaps, it should be treated as scalable and scaled to the desired size. Metrics
                // based on y_ppem need to be rescaled for the adjusted size.
                if face_handle.color() {
                    em_height = self.requested_face_size.to_f64_px();
                    let adjust_scale = em_height / (freetype_metrics.y_ppem as f64);
                    max_advance *= adjust_scale;
                    max_descent *= adjust_scale;
                    max_ascent *= adjust_scale;
                    line_height *= adjust_scale;
                }
                let units_per_em = unsafe { (*head).units_per_em } as f64;
                y_scale = em_height / units_per_em;
            }
        }

        // 'leading' is supposed to be the vertical distance between two baselines,
        // reflected by the height attribute in freetype. On OS X (w/ CTFont),
        // leading represents the distance between the bottom of a line descent to
        // the top of the next line's ascent or: (line_height - ascent - descent),
        // see http://stackoverflow.com/a/5635981 for CTFont implementation.
        // Convert using a formula similar to what CTFont returns for consistency.
        let leading = line_height - (max_ascent + max_descent);

        let underline_size = face.underline_thickness as f64 * y_scale;
        let underline_offset = face.underline_position as f64 * y_scale + 0.5;

        // The default values for strikeout size and offset. Use OpenType spec's suggested position
        // for Roman font as the default for offset.
        let mut strikeout_size = underline_size;
        let mut strikeout_offset = em_height * 409.0 / 2048.0 + 0.5 * strikeout_size;

        // CSS 2.1, section 4.3.2 Lengths: "In the cases where it is
        // impossible or impractical to determine the x-height, a value of
        // 0.5em should be used."
        let mut x_height = 0.5 * em_height;
        let mut average_advance = 0.0;
        if let Some(os2) = face_handle.os2_table() {
            if !os2.y_strikeout_size.is_zero() && !os2.y_strikeout_position.is_zero() {
                strikeout_size = os2.y_strikeout_size as f64 * y_scale;
                strikeout_offset = os2.y_strikeout_position as f64 * y_scale;
            }
            if !os2.sx_height.is_zero() {
                x_height = os2.sx_height as f64 * y_scale;
            }

            if !os2.x_average_char_width.is_zero() {
                average_advance = fixed_26_dot_6_to_float(unsafe {
                    FT_MulFix(
                        os2.x_average_char_width as FT_F26Dot6,
                        freetype_metrics.x_scale,
                    )
                });
            }
        }

        if average_advance.is_zero() {
            average_advance = self
                .glyph_index('0')
                .and_then(|idx| self.glyph_h_advance(idx))
                .map_or(max_advance, |advance| advance * y_scale);
        }

        let zero_horizontal_advance = self
            .glyph_index('0')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let ic_horizontal_advance = self
            .glyph_index('\u{6C34}')
            .and_then(|idx| self.glyph_h_advance(idx))
            .map(Au::from_f64_px);
        let space_advance = self
            .glyph_index(' ')
            .and_then(|idx| self.glyph_h_advance(idx))
            .unwrap_or(average_advance);

        FontMetrics {
            underline_size: Au::from_f64_px(underline_size),
            underline_offset: Au::from_f64_px(underline_offset),
            strikeout_size: Au::from_f64_px(strikeout_size),
            strikeout_offset: Au::from_f64_px(strikeout_offset),
            leading: Au::from_f64_px(leading),
            x_height: Au::from_f64_px(x_height),
            em_size: Au::from_f64_px(em_height),
            ascent: Au::from_f64_px(max_ascent),
            descent: Au::from_f64_px(max_descent),
            max_advance: Au::from_f64_px(max_advance),
            average_advance: Au::from_f64_px(average_advance),
            line_gap: Au::from_f64_px(line_height),
            zero_horizontal_advance,
            ic_horizontal_advance,
            space_advance: Au::from_f64_px(space_advance),
        }
    }

    fn table_for_tag(&self, tag: FontTableTag) -> Option<FontTable> {
        let face_handle = self.face_handle.lock();
        face_handle.table_for_tag(tag)
    }

    fn typographic_bounds(&self, glyph_id: GlyphId) -> Rect<f32> {
        let face_handle = self.face_handle.lock();
        let face = face_handle.face;
        assert!(!face.is_null());

        let load_flags = FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING;
        let result = unsafe { FT_Load_Glyph(face, glyph_id as FT_UInt, load_flags) };
        if 0 != result {
            debug!("Unable to load glyph {}. reason: {:?}", glyph_id, result);
            return Rect::default();
        }

        let metrics = unsafe { &(*(*face).glyph).metrics };

        Rect::new(
            Point2D::new(
                metrics.horiBearingX as f32,
                (metrics.horiBearingY - metrics.height) as f32,
            ),
            Size2D::new(metrics.width as f32, metrics.height as f32),
        ) * (1. / 64.)
    }

    fn webrender_font_instance_flags(&self) -> FontInstanceFlags {
        // On other platforms, we only pass this when we know that we are loading a font with
        // color characters, but not passing this flag simply *prevents* WebRender from
        // loading bitmaps. There's no harm to always passing it.
        FontInstanceFlags::EMBEDDED_BITMAPS
    }
}

#[repr(C)]
struct TtHeader {
    table_version: FT_Fixed,
    font_revision: FT_Fixed,

    checksum_adjust: FT_Long,
    magic_number: FT_Long,

    flags: FT_UShort,
    units_per_em: FT_UShort,

    created: [FT_ULong; 2],
    modified: [FT_ULong; 2],

    x_min: FT_Short,
    y_min: FT_Short,
    x_max: FT_Short,
    y_max: FT_Short,

    mac_style: FT_UShort,
    lowest_rec_ppem: FT_UShort,

    font_direction: FT_Short,
    index_to_loc_format: FT_Short,
    glyph_data_format: FT_Short,
}
