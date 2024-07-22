/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::convert::TryInto;
use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;

use base::text::{UnicodeBlock, UnicodeBlockMethod};
use fontconfig_sys::constants::{
    FC_FAMILY, FC_FILE, FC_FONTFORMAT, FC_INDEX, FC_SLANT, FC_SLANT_ITALIC, FC_SLANT_OBLIQUE,
    FC_WEIGHT, FC_WEIGHT_BOLD, FC_WEIGHT_EXTRABLACK, FC_WEIGHT_REGULAR, FC_WIDTH,
    FC_WIDTH_CONDENSED, FC_WIDTH_EXPANDED, FC_WIDTH_EXTRACONDENSED, FC_WIDTH_EXTRAEXPANDED,
    FC_WIDTH_NORMAL, FC_WIDTH_SEMICONDENSED, FC_WIDTH_SEMIEXPANDED, FC_WIDTH_ULTRACONDENSED,
    FC_WIDTH_ULTRAEXPANDED,
};
use fontconfig_sys::{
    FcChar8, FcConfigGetCurrent, FcConfigGetFonts, FcConfigSubstitute, FcDefaultSubstitute,
    FcFontMatch, FcFontSetDestroy, FcFontSetList, FcMatchPattern, FcNameParse, FcObjectSetAdd,
    FcObjectSetCreate, FcObjectSetDestroy, FcPattern, FcPatternAddString, FcPatternCreate,
    FcPatternDestroy, FcPatternGetInteger, FcPatternGetString, FcResultMatch, FcSetSystem,
};
use libc::{c_char, c_int};
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{FontStretch, FontStyle, FontWeight};
use style::Atom;
use unicode_script::Script;

use super::c_str_to_string;
use crate::font::map_platform_values_to_style_values;
use crate::font_template::{FontTemplate, FontTemplateDescriptor};
use crate::platform::add_noto_fallback_families;
use crate::{EmojiPresentationPreference, FallbackFontSelectionOptions, LowercaseFontFamilyName};

/// An identifier for a local font on systems using Freetype.
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    /// The path to the font.
    pub path: Atom,
    /// The variation index within the font.
    pub variation_index: i32,
}

impl LocalFontIdentifier {
    pub(crate) fn index(&self) -> u32 {
        self.variation_index.try_into().unwrap()
    }

    pub(crate) fn read_data_from_file(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        File::open(Path::new(&*self.path))
            .expect("Couldn't open font file!")
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    }
}

pub fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    unsafe {
        let config = FcConfigGetCurrent();
        let font_set = FcConfigGetFonts(config, FcSetSystem);
        for i in 0..((*font_set).nfont as isize) {
            let font = (*font_set).fonts.offset(i);
            let mut family: *mut FcChar8 = ptr::null_mut();
            let mut format: *mut FcChar8 = ptr::null_mut();
            let mut v: c_int = 0;
            if FcPatternGetString(*font, FC_FONTFORMAT.as_ptr() as *mut c_char, v, &mut format) !=
                FcResultMatch
            {
                continue;
            }

            // Skip bitmap fonts. They aren't supported by FreeType.
            let fontformat = c_str_to_string(format as *const c_char);
            if fontformat != "TrueType" && fontformat != "CFF" && fontformat != "Type 1" {
                continue;
            }

            while FcPatternGetString(*font, FC_FAMILY.as_ptr() as *mut c_char, v, &mut family) ==
                FcResultMatch
            {
                let family_name = c_str_to_string(family as *const c_char);
                callback(family_name);
                v += 1;
            }
        }
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
{
    unsafe {
        let config = FcConfigGetCurrent();
        let mut font_set = FcConfigGetFonts(config, FcSetSystem);
        let font_set_array_ptr = &mut font_set;
        let pattern = FcPatternCreate();
        assert!(!pattern.is_null());
        let family_name_cstr: CString = CString::new(family_name).unwrap();
        let ok = FcPatternAddString(
            pattern,
            FC_FAMILY.as_ptr() as *mut c_char,
            family_name_cstr.as_ptr() as *const FcChar8,
        );
        assert_ne!(ok, 0);

        let object_set = FcObjectSetCreate();
        assert!(!object_set.is_null());

        FcObjectSetAdd(object_set, FC_FILE.as_ptr() as *mut c_char);
        FcObjectSetAdd(object_set, FC_INDEX.as_ptr() as *mut c_char);
        FcObjectSetAdd(object_set, FC_WEIGHT.as_ptr() as *mut c_char);
        FcObjectSetAdd(object_set, FC_SLANT.as_ptr() as *mut c_char);
        FcObjectSetAdd(object_set, FC_WIDTH.as_ptr() as *mut c_char);

        let matches = FcFontSetList(config, font_set_array_ptr, 1, pattern, object_set);
        debug!("Found {} variations for {}", (*matches).nfont, family_name);

        for variation_index in 0..((*matches).nfont as isize) {
            let font = (*matches).fonts.offset(variation_index);

            let mut path: *mut FcChar8 = ptr::null_mut();
            let result = FcPatternGetString(*font, FC_FILE.as_ptr() as *mut c_char, 0, &mut path);
            assert_eq!(result, FcResultMatch);

            let mut index: libc::c_int = 0;
            let result =
                FcPatternGetInteger(*font, FC_INDEX.as_ptr() as *mut c_char, 0, &mut index);
            assert_eq!(result, FcResultMatch);

            let Some(weight) = font_weight_from_fontconfig_pattern(*font) else {
                continue;
            };
            let Some(stretch) = font_stretch_from_fontconfig_pattern(*font) else {
                continue;
            };
            let Some(style) = font_style_from_fontconfig_pattern(*font) else {
                continue;
            };

            let local_font_identifier = LocalFontIdentifier {
                path: Atom::from(c_str_to_string(path as *const c_char)),
                variation_index: index as i32,
            };
            let descriptor = FontTemplateDescriptor::new(weight, stretch, style);

            callback(FontTemplate::new_for_local_font(
                local_font_identifier,
                descriptor,
            ))
        }

        FcFontSetDestroy(matches);
        FcPatternDestroy(pattern);
        FcObjectSetDestroy(object_set);
    }
}

// Based on gfxPlatformGtk::GetCommonFallbackFonts() in Gecko
pub fn fallback_font_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
    let mut families = Vec::new();
    if options.presentation_preference == EmojiPresentationPreference::Emoji {
        families.push("Noto Color Emoji");
    }

    add_noto_fallback_families(options, &mut families);

    if matches!(
        Script::from(options.character),
        Script::Bopomofo | Script::Han
    ) {
        families.push("WenQuanYi Micro Hei");
    }

    if let Some(block) = options.character.block() {
        match block {
            UnicodeBlock::HalfwidthandFullwidthForms |
            UnicodeBlock::EnclosedIdeographicSupplement => families.push("WenQuanYi Micro Hei"),
            UnicodeBlock::Hiragana |
            UnicodeBlock::Katakana |
            UnicodeBlock::KatakanaPhoneticExtensions => {
                families.push("TakaoPGothic");
            },
            _ => {},
        }
    }

    families.push("DejaVu Serif");
    families.push("FreeSerif");
    families.push("DejaVu Sans");
    families.push("DejaVu Sans Mono");
    families.push("FreeSans");
    families.push("Symbola");
    families.push("Droid Sans Fallback");

    families
}

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
    let generic_string = match generic {
        GenericFontFamily::None | GenericFontFamily::Serif => "serif",
        GenericFontFamily::SansSerif => "sans-serif",
        GenericFontFamily::Monospace => "monospace",
        GenericFontFamily::Cursive => "cursive",
        GenericFontFamily::Fantasy => "fantasy",
        GenericFontFamily::SystemUi => "sans-serif",
    };

    let generic_name_c = CString::new(generic_string).unwrap();
    let generic_name_ptr = generic_name_c.as_ptr();

    unsafe {
        let pattern = FcNameParse(generic_name_ptr as *mut FcChar8);
        FcConfigSubstitute(ptr::null_mut(), pattern, FcMatchPattern);
        FcDefaultSubstitute(pattern);

        let mut result = 0;
        let family_match = FcFontMatch(ptr::null_mut(), pattern, &mut result);

        if result == FcResultMatch {
            let mut match_string: *mut FcChar8 = ptr::null_mut();
            FcPatternGetString(
                family_match,
                FC_FAMILY.as_ptr() as *mut c_char,
                0,
                &mut match_string,
            );
            let family_name = c_str_to_string(match_string as *const c_char);

            FcPatternDestroy(family_match);
            FcPatternDestroy(pattern);

            return family_name.into();
        }

        FcPatternDestroy(family_match);
        FcPatternDestroy(pattern);
    }

    match generic {
        GenericFontFamily::None | GenericFontFamily::Serif => "Noto Serif",
        GenericFontFamily::SansSerif => "Noto Sans",
        GenericFontFamily::Monospace => "Deja Vu Sans Mono",
        GenericFontFamily::Cursive => "Comic Sans MS",
        GenericFontFamily::Fantasy => "Impact",
        GenericFontFamily::SystemUi => "Noto Sans",
    }
    .into()
}

fn font_style_from_fontconfig_pattern(pattern: *mut FcPattern) -> Option<FontStyle> {
    let mut slant: c_int = 0;
    unsafe {
        if FcResultMatch != FcPatternGetInteger(pattern, FC_SLANT.as_ptr(), 0, &mut slant) {
            return None;
        }
    }
    Some(match slant {
        FC_SLANT_ITALIC => FontStyle::ITALIC,
        FC_SLANT_OBLIQUE => FontStyle::OBLIQUE,
        _ => FontStyle::NORMAL,
    })
}

fn font_stretch_from_fontconfig_pattern(pattern: *mut FcPattern) -> Option<FontStretch> {
    let mut width: c_int = 0;
    unsafe {
        if FcResultMatch != FcPatternGetInteger(pattern, FC_WIDTH.as_ptr(), 0, &mut width) {
            return None;
        }
    }
    let mapping = [
        (FC_WIDTH_ULTRACONDENSED as f64, 0.5),
        (FC_WIDTH_EXTRACONDENSED as f64, 0.625),
        (FC_WIDTH_CONDENSED as f64, 0.75),
        (FC_WIDTH_SEMICONDENSED as f64, 0.875),
        (FC_WIDTH_NORMAL as f64, 1.0),
        (FC_WIDTH_SEMIEXPANDED as f64, 1.125),
        (FC_WIDTH_EXPANDED as f64, 1.25),
        (FC_WIDTH_EXTRAEXPANDED as f64, 1.50),
        (FC_WIDTH_ULTRAEXPANDED as f64, 2.00),
    ];

    let mapped_width = map_platform_values_to_style_values(&mapping, width as f64);
    Some(FontStretch::from_percentage(mapped_width as f32))
}

fn font_weight_from_fontconfig_pattern(pattern: *mut FcPattern) -> Option<FontWeight> {
    let mut weight: c_int = 0;
    unsafe {
        let result = FcPatternGetInteger(pattern, FC_WEIGHT.as_ptr(), 0, &mut weight);
        if result != FcResultMatch {
            return None;
        }
    }

    let mapping = [
        (0., 0.),
        (FC_WEIGHT_REGULAR as f64, 400_f64),
        (FC_WEIGHT_BOLD as f64, 700_f64),
        (FC_WEIGHT_EXTRABLACK as f64, 1000_f64),
    ];

    let mapped_weight = map_platform_values_to_style_values(&mapping, weight as f64);
    Some(FontWeight::from_float(mapped_weight as f32))
}
