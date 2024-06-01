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
use style::values::computed::{FontStretch, FontStyle, FontWeight};
use style::Atom;
use unicode_script::Script;

use super::c_str_to_string;
use crate::font::map_platform_values_to_style_values;
use crate::font_template::{FontTemplate, FontTemplateDescriptor};
use crate::text::FallbackFontSelectionOptions;

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

pub fn system_default_family(generic_name: &str) -> Option<String> {
    let generic_name_c = CString::new(generic_name).unwrap();
    let generic_name_ptr = generic_name_c.as_ptr();

    unsafe {
        let pattern = FcNameParse(generic_name_ptr as *mut FcChar8);

        FcConfigSubstitute(ptr::null_mut(), pattern, FcMatchPattern);
        FcDefaultSubstitute(pattern);

        let mut result = 0;
        let family_match = FcFontMatch(ptr::null_mut(), pattern, &mut result);

        let family_name = if result == FcResultMatch {
            let mut match_string: *mut FcChar8 = ptr::null_mut();
            FcPatternGetString(
                family_match,
                FC_FAMILY.as_ptr() as *mut c_char,
                0,
                &mut match_string,
            );
            let result = c_str_to_string(match_string as *const c_char);
            FcPatternDestroy(family_match);
            Some(result)
        } else {
            None
        };

        FcPatternDestroy(pattern);
        family_name
    }
}

pub static SANS_SERIF_FONT_FAMILY: &str = "DejaVu Sans";

// Based on gfxPlatformGtk::GetCommonFallbackFonts() in Gecko
pub fn fallback_font_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
    let mut families = Vec::new();
    if options.prefer_emoji_presentation {
        families.push("Noto Color Emoji");
    }

    let add_chinese_families = |families: &mut Vec<&str>| {
        // TODO: Need to differentiate between traditional and simplified Han here!
        families.push("Noto Sans CJK HK");
        families.push("Noto Sans CJK SC");
        families.push("Noto Sans CJK TC");
        families.push("Noto Sans HK");
        families.push("Noto Sans SC");
        families.push("Noto Sans TC");
        families.push("WenQuanYi Micro Hei");
    };

    match Script::from(options.character) {
        // In most cases, COMMON and INHERITED characters will be merged into
        // their context, but if they occur without any specific script context
        // we'll just try common default fonts here.
        Script::Common | Script::Inherited | Script::Latin | Script::Cyrillic | Script::Greek => {
            families.push("Noto Sans");
        },
        // CJK-related script codes are a bit troublesome because of unification;
        // we'll probably just get HAN much of the time, so the choice of which
        // language font to try for fallback is rather arbitrary. Usually, though,
        // we hope that font prefs will have handled this earlier.
        Script::Bopomofo | Script::Han => add_chinese_families(&mut families),
        Script::Hanifi_Rohingya => families.push("Noto Sans Hanifi Rohingya"),
        Script::Wancho => families.push("Noto Sans Wancho"),
        _ => {},
    }

    if let Some(block) = options.character.block() {
        match block {
            UnicodeBlock::HalfwidthandFullwidthForms |
            UnicodeBlock::EnclosedIdeographicSupplement => add_chinese_families(&mut families),
            UnicodeBlock::Adlam => families.push("Noto Sans Adlam"),
            UnicodeBlock::Ahom => families.push("Noto Serif Ahom"),
            UnicodeBlock::AnatolianHieroglyphs => families.push("Noto Sans AnatoHiero"),
            UnicodeBlock::Arabic |
            UnicodeBlock::ArabicExtendedA |
            UnicodeBlock::ArabicPresentationFormsA |
            UnicodeBlock::ArabicPresentationFormsB => {
                families.push("Noto Sans Arabic");
                families.push("Noto Naskh Arabic");
            },
            UnicodeBlock::ArabicMathematicalAlphabeticSymbols => {
                families.push("Noto Sans Math");
            },
            UnicodeBlock::Armenian => families.push("Noto Sans Armenian"),
            UnicodeBlock::Avestan => families.push("Noto Sans Avestan"),
            UnicodeBlock::Balinese => families.push("Noto Sans Balinese"),
            UnicodeBlock::Bamum | UnicodeBlock::BamumSupplement => families.push("Noto Sans Bamum"),
            UnicodeBlock::BassaVah => families.push("Noto Sans Bassa Vah"),
            UnicodeBlock::Batak => families.push("Noto Sans Batak"),
            UnicodeBlock::Bengali => families.push("Noto Sans Bengali"),
            UnicodeBlock::Bhaiksuki => families.push("Noto Sans Bhaiksuki"),
            UnicodeBlock::Brahmi => families.push("Noto Sans Brahmi"),
            UnicodeBlock::BraillePatterns => {
                // These characters appear to be in DejaVu Serif.
            },
            UnicodeBlock::Buginese => families.push("Noto Sans Buginese"),
            UnicodeBlock::Buhid => families.push("Noto Sans Buhid"),
            UnicodeBlock::Carian => families.push("Noto Sans Carian"),
            UnicodeBlock::CaucasianAlbanian => families.push("Noto Sans Caucasian Albanian"),
            UnicodeBlock::Chakma => families.push("Noto Sans Chakma"),
            UnicodeBlock::Cham => families.push("Noto Sans Cham"),
            UnicodeBlock::Cherokee | UnicodeBlock::CherokeeSupplement => {
                families.push("Noto Sans Cherokee")
            },
            UnicodeBlock::Coptic => families.push("Noto Sans Coptic"),
            UnicodeBlock::Cuneiform | UnicodeBlock::CuneiformNumbersandPunctuation => {
                families.push("Noto Sans Cuneiform")
            },
            UnicodeBlock::CypriotSyllabary => families.push("Noto Sans Cypriot"),
            UnicodeBlock::Deseret => families.push("Noto Sans Deseret"),
            UnicodeBlock::Devanagari |
            UnicodeBlock::DevanagariExtended |
            UnicodeBlock::CommonIndicNumberForms => families.push("Noto Sans Devanagari"),
            UnicodeBlock::Duployan => families.push("Noto Sans Duployan"),
            UnicodeBlock::EgyptianHieroglyphs => families.push("Noto Sans Egyptian Hieroglyphs"),
            UnicodeBlock::Elbasan => families.push("Noto Sans Elbasan"),
            UnicodeBlock::Ethiopic |
            UnicodeBlock::EthiopicExtended |
            UnicodeBlock::EthiopicExtendedA |
            UnicodeBlock::EthiopicSupplement => families.push("Noto Sans Ethiopic"),
            UnicodeBlock::Georgian | UnicodeBlock::GeorgianSupplement => {
                families.push("Noto Sans Georgian")
            },
            UnicodeBlock::Glagolitic | UnicodeBlock::GlagoliticSupplement => {
                families.push("Noto Sans Glagolitic")
            },
            UnicodeBlock::Gothic => families.push("Noto Sans Gothic"),
            UnicodeBlock::Grantha => families.push("Noto Sans Grantha"),
            UnicodeBlock::Gujarati => families.push("Noto Sans Gujarati"),
            UnicodeBlock::Gurmukhi => families.push("Noto Sans Gurmukhi"),
            UnicodeBlock::HangulCompatibilityJamo |
            UnicodeBlock::HangulJamo |
            UnicodeBlock::HangulJamoExtendedA |
            UnicodeBlock::HangulJamoExtendedB |
            UnicodeBlock::HangulSyllables => {
                families.push("Noto Sans KR");
                families.push("Noto Sans CJK KR");
            },
            UnicodeBlock::Hanunoo => families.push("Noto Sans Hanunoo"),
            UnicodeBlock::Hatran => families.push("Noto Sans Hatran"),
            UnicodeBlock::Hebrew => families.push("Noto Sans Hebrew"),
            UnicodeBlock::Hiragana |
            UnicodeBlock::Katakana |
            UnicodeBlock::KatakanaPhoneticExtensions => {
                families.push("TakaoPGothic");
                families.push("Noto Sans JP");
                families.push("Noto Sans CJK JP");
            },
            UnicodeBlock::ImperialAramaic => families.push("Noto Sans Imperial Aramaic"),
            UnicodeBlock::InscriptionalPahlavi => families.push("Noto Sans Inscriptional Pahlavi"),
            UnicodeBlock::InscriptionalParthian => {
                families.push("Noto Sans Inscriptional Parthian")
            },
            UnicodeBlock::Javanese => families.push("Noto Sans Javanese"),
            UnicodeBlock::Kaithi => families.push("Noto Sans Kaithi"),
            UnicodeBlock::Kannada => families.push("Noto Sans Kannada"),
            UnicodeBlock::KayahLi => families.push("Noto Sans Kayah Li"),
            UnicodeBlock::Kharoshthi => families.push("Noto Sans Kharoshthi"),
            UnicodeBlock::Khmer | UnicodeBlock::KhmerSymbols => families.push("Noto Sans Khmer"),
            UnicodeBlock::Khojki => families.push("Noto Sans Khojki"),
            UnicodeBlock::Khudawadi => families.push("Noto Sans Khudawadi"),
            UnicodeBlock::Lao => families.push("Noto Sans Lao"),
            UnicodeBlock::Lepcha => families.push("Noto Sans Lepcha"),
            UnicodeBlock::Limbu => families.push("Noto Sans Limbu"),
            UnicodeBlock::LinearA => families.push("Noto Sans Linear A"),
            UnicodeBlock::LinearBIdeograms | UnicodeBlock::LinearBSyllabary => {
                families.push("Noto Sans Linear B")
            },
            UnicodeBlock::Lisu => families.push("Noto Sans Lisu"),
            UnicodeBlock::Lycian => families.push("Noto Sans Lycian"),
            UnicodeBlock::Lydian => families.push("Noto Sans Lydian"),
            UnicodeBlock::Mahajani => families.push("Noto Sans Mahajani"),
            UnicodeBlock::Malayalam => families.push("Noto Sans Malayalam"),
            UnicodeBlock::Mandaic => families.push("Noto Sans Mandaic"),
            UnicodeBlock::Manichaean => families.push("Noto Sans Manichaean"),
            UnicodeBlock::Marchen => families.push("Noto Sans Marchen"),
            UnicodeBlock::MeeteiMayek | UnicodeBlock::MeeteiMayekExtensions => {
                families.push("Noto Sans Meetei Mayek")
            },
            UnicodeBlock::MendeKikakui => families.push("Noto Sans Mende Kikakui"),
            UnicodeBlock::MeroiticCursive | UnicodeBlock::MeroiticHieroglyphs => {
                families.push("Noto Sans Meroitic")
            },
            UnicodeBlock::Miao => families.push("Noto Sans Miao"),
            UnicodeBlock::Modi => families.push("Noto Sans Modi"),
            UnicodeBlock::Mongolian | UnicodeBlock::MongolianSupplement => {
                families.push("Noto Sans Mongolian")
            },
            UnicodeBlock::Mro => families.push("Noto Sans Mro"),
            UnicodeBlock::Multani => families.push("Noto Sans Multani"),
            UnicodeBlock::MusicalSymbols => families.push("Noto Music"),
            UnicodeBlock::Myanmar |
            UnicodeBlock::MyanmarExtendedA |
            UnicodeBlock::MyanmarExtendedB => families.push("Noto Sans Myanmar"),
            UnicodeBlock::NKo => families.push("Noto Sans NKo"),
            UnicodeBlock::Nabataean => families.push("Noto Sans Nabataean"),
            UnicodeBlock::NewTaiLue => families.push("Noto Sans New Tai Lue"),
            UnicodeBlock::Newa => families.push("Noto Sans Newa"),
            UnicodeBlock::Ogham => families.push("Noto Sans Ogham"),
            UnicodeBlock::OlChiki => families.push("Noto Sans Ol Chiki"),
            UnicodeBlock::OldHungarian => families.push("Noto Sans Old Hungarian"),
            UnicodeBlock::OldItalic => families.push("Noto Sans Old Italic"),
            UnicodeBlock::OldNorthArabian => families.push("Noto Sans Old North Arabian"),
            UnicodeBlock::OldPermic => families.push("Noto Sans Old Permic"),
            UnicodeBlock::OldPersian => families.push("Noto Sans Old Persian"),
            UnicodeBlock::OldSouthArabian => families.push("Noto Sans Old South Arabian"),
            UnicodeBlock::OldTurkic => families.push("Noto Sans Old Turkic"),
            UnicodeBlock::Oriya => families.push("Noto Sans Oriya"),
            UnicodeBlock::Osage => families.push("Noto Sans Osage"),
            UnicodeBlock::Osmanya => families.push("Noto Sans Osmanya"),
            UnicodeBlock::PahawhHmong => families.push("Noto Sans Pahawh Hmong"),
            UnicodeBlock::Palmyrene => families.push("Noto Sans Palmyrene"),
            UnicodeBlock::PauCinHau => families.push("Noto Sans Pau Cin Hau"),
            UnicodeBlock::Phagspa => families.push("Noto Sans PhagsPa"),
            UnicodeBlock::Phoenician => families.push("Noto Sans Phoenician"),
            UnicodeBlock::PsalterPahlavi => families.push("Noto Sans Psalter Pahlavi"),
            UnicodeBlock::Rejang => families.push("Noto Sans Rejang"),
            UnicodeBlock::Runic => families.push("Noto Sans Runic"),
            UnicodeBlock::Samaritan => families.push("Noto Sans Samaritan"),
            UnicodeBlock::Saurashtra => families.push("Noto Sans Saurashtra"),
            UnicodeBlock::Sharada => families.push("Noto Sans Sharada"),
            UnicodeBlock::Shavian => families.push("Noto Sans Shavian"),
            UnicodeBlock::Siddham => families.push("Noto Sans Siddham"),
            UnicodeBlock::Sinhala | UnicodeBlock::SinhalaArchaicNumbers => {
                families.push("Noto Sans Sinhala")
            },
            UnicodeBlock::SoraSompeng => families.push("Noto Sans Sora Sompeng"),
            UnicodeBlock::Sundanese => families.push("Noto Sans Sundanese"),
            UnicodeBlock::SuttonSignWriting => families.push("Noto Sans SignWrit"),
            UnicodeBlock::SylotiNagri => families.push("Noto Sans Syloti Nagri"),
            UnicodeBlock::Syriac => families.push("Noto Sans Syriac"),
            UnicodeBlock::Tagalog => families.push("Noto Sans Tagalog"),
            UnicodeBlock::Tagbanwa => families.push("Noto Sans Tagbanwa"),
            UnicodeBlock::TaiLe => families.push("Noto Sans Tai Le"),
            UnicodeBlock::TaiTham => families.push("Noto Sans Tai Tham"),
            UnicodeBlock::TaiViet => families.push("Noto Sans Tai Viet"),
            UnicodeBlock::Takri => families.push("Noto Sans Takri"),
            UnicodeBlock::Tamil => families.push("Noto Sans Tamil"),
            UnicodeBlock::Tangut |
            UnicodeBlock::TangutComponents |
            UnicodeBlock::IdeographicSymbolsandPunctuation => families.push("Noto Serif Tangut"),
            UnicodeBlock::Telugu => families.push("Noto Sans Telugu"),
            UnicodeBlock::Thaana => {
                families.push("Noto Sans Thaana");
            },
            UnicodeBlock::Thai => families.push("Noto Sans Thai"),
            UnicodeBlock::Tibetan => families.push("Noto Serif Tibetan"),
            UnicodeBlock::Tifinagh => families.push("Noto Sans Tifinagh"),
            UnicodeBlock::Tirhuta => families.push("Noto Sans Tirhuta"),
            UnicodeBlock::Ugaritic => families.push("Noto Sans Ugaritic"),
            UnicodeBlock::UnifiedCanadianAboriginalSyllabics |
            UnicodeBlock::UnifiedCanadianAboriginalSyllabicsExtended => {
                families.push("Noto Sans Canadian Aboriginal")
            },
            UnicodeBlock::Vai => families.push("Noto Sans Vai"),
            UnicodeBlock::WarangCiti => families.push("Noto Sans Warang Citi"),
            UnicodeBlock::YiSyllables | UnicodeBlock::YiRadicals => {
                families.push("Noto Sans Yi");
            },
            _ => {},
        }
    }

    families.push("DejaVu Serif");
    families.push("FreeSerif");
    families.push("DejaVu Sans");
    families.push("DejaVu Sans Mono");
    families.push("FreeSans");
    families.push("Noto Sans Symbols");
    families.push("Noto Sans Symbols2");
    families.push("Symbola");
    families.push("Droid Sans Fallback");

    families
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
