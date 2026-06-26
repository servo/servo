/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::{Arc, OnceLock};

use dwrote::{Font, FontCollection, FontFamily, FontStretch, FontStyle};
use fonts_traits::LocalFontIdentifier;
use servo_base::text::{UnicodeBlock, UnicodeBlockMethod, unicode_plane};
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{FontStyle as StyleFontStyle, FontWeight as StyleFontWeight};
use style::values::specified::font::FontStretchKeyword;
use winapi::um::dwrite::{IDWriteFontFamily, IDWriteLocalizedStrings};

use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontIdentifier, FontTemplate,
    FontTemplateDescriptor, LowercaseFontFamilyName,
};

/// Read every localized name stored in an `IDWriteFontFamily`'s
/// `IDWriteLocalizedStrings`. `dwrote::FontFamily::family_name()` only returns
/// the name for the user's system locale, so on (for example) a zh-CN install
/// it yields "微软雅黑" and hides "Microsoft YaHei" from the rest of Servo.
/// DirectWrite's `FindFamilyName` on the system collection in turn only
/// resolves the English WWS name, so without this both directions of lookup
/// fail and CJK fallback silently misses Microsoft YaHei.
unsafe fn all_family_names(family_ptr: *mut IDWriteFontFamily) -> Vec<String> {
    let mut strings: *mut IDWriteLocalizedStrings = std::ptr::null_mut();
    if unsafe { (*family_ptr).GetFamilyNames(&mut strings) } != 0 || strings.is_null() {
        return Vec::new();
    }
    let count = unsafe { (*strings).GetCount() };
    let mut names = Vec::with_capacity(count as usize);
    for index in 0..count {
        let mut len: u32 = 0;
        if unsafe { (*strings).GetStringLength(index, &mut len) } != 0 {
            continue;
        }
        let mut buf: Vec<u16> = vec![0u16; (len as usize) + 1];
        if unsafe { (*strings).GetString(index, buf.as_mut_ptr(), len + 1) } != 0 {
            continue;
        }
        buf.truncate(len as usize);
        if let Ok(name) = OsString::from_wide(&buf).into_string() {
            names.push(name);
        }
    }
    unsafe { (*strings).Release() };
    names
}

/// Lowercased family name (any locale) → index into the system
/// `FontCollection`. Built once on first use so subsequent lookups are O(1).
fn family_name_to_index_cache() -> &'static HashMap<String, u32> {
    static CACHE: OnceLock<HashMap<String, u32>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let system_fc = FontCollection::system();
        let count = system_fc.get_font_family_count();
        let mut map = HashMap::with_capacity(count as usize * 2);
        for index in 0..count {
            let Ok(family) = system_fc.font_family(index) else {
                continue;
            };
            let names = unsafe { all_family_names(family.as_ptr()) };
            for name in names {
                map.entry(name.to_lowercase()).or_insert(index);
            }
        }
        map
    })
}

fn find_family_by_any_name(collection: &FontCollection, family_name: &str) -> Option<FontFamily> {
    if let Ok(Some(family)) = collection.font_family_by_name(family_name) {
        return Some(family);
    }
    let index = *family_name_to_index_cache().get(&family_name.to_lowercase())?;
    collection.font_family(index).ok()
}

pub(crate) fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    let system_fc = FontCollection::system();
    let count = system_fc.get_font_family_count();
    for index in 0..count {
        let Ok(family) = system_fc.font_family(index) else {
            continue;
        };
        let names = unsafe { all_family_names(family.as_ptr()) };
        for name in names {
            callback(name);
        }
    }
}

pub(crate) fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
{
    let system_fc = FontCollection::system();
    let Some(family) = find_family_by_any_name(&system_fc, family_name) else {
        return;
    };
    let count = family.get_font_count();
    for i in 0..count {
        let Ok(font) = family.font(i) else {
            continue;
        };
        let template_descriptor = font_template_descriptor_from_font(&font);
        let local_font_identifier = LocalFontIdentifier {
            font_descriptor: Arc::new(font.to_descriptor()),
        };
        callback(FontTemplate::new(
            FontIdentifier::Local(local_font_identifier),
            template_descriptor,
            None,
            None,
        ))
    }
}

// Based on gfxWindowsPlatform::GetCommonFallbackFonts() in Gecko
pub fn fallback_font_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
    let mut families = Vec::new();
    if options.presentation_preference == EmojiPresentationPreference::Emoji {
        families.push("Segoe UI Emoji");
    }

    families.push("Arial");
    match unicode_plane(options.character) {
        // https://en.wikipedia.org/wiki/Plane_(Unicode)#Basic_Multilingual_Plane
        0 => {
            if let Some(block) = options.character.block() {
                match block {
                    UnicodeBlock::CyrillicSupplement |
                    UnicodeBlock::Armenian |
                    UnicodeBlock::Hebrew => {
                        families.push("Estrangelo Edessa");
                        families.push("Cambria");
                    },

                    UnicodeBlock::Arabic | UnicodeBlock::ArabicSupplement => {
                        families.push("Microsoft Uighur");
                    },

                    UnicodeBlock::Syriac => {
                        families.push("Estrangelo Edessa");
                    },

                    UnicodeBlock::Thaana => {
                        families.push("MV Boli");
                    },

                    UnicodeBlock::NKo => {
                        families.push("Ebrima");
                    },

                    UnicodeBlock::Devanagari | UnicodeBlock::Bengali => {
                        families.push("Nirmala UI");
                        families.push("Utsaah");
                        families.push("Aparajita");
                    },

                    UnicodeBlock::Gurmukhi |
                    UnicodeBlock::Gujarati |
                    UnicodeBlock::Oriya |
                    UnicodeBlock::Tamil |
                    UnicodeBlock::Telugu |
                    UnicodeBlock::Kannada |
                    UnicodeBlock::Malayalam |
                    UnicodeBlock::Sinhala |
                    UnicodeBlock::Lepcha |
                    UnicodeBlock::OlChiki |
                    UnicodeBlock::CyrillicExtendedC |
                    UnicodeBlock::SundaneseSupplement |
                    UnicodeBlock::VedicExtensions => {
                        families.push("Nirmala UI");
                    },

                    UnicodeBlock::Thai => {
                        families.push("Leelawadee UI");
                    },

                    UnicodeBlock::Lao => {
                        families.push("Lao UI");
                    },

                    UnicodeBlock::Myanmar |
                    UnicodeBlock::MyanmarExtendedA |
                    UnicodeBlock::MyanmarExtendedB => {
                        families.push("Myanmar Text");
                    },

                    UnicodeBlock::HangulJamo |
                    UnicodeBlock::HangulJamoExtendedA |
                    UnicodeBlock::HangulSyllables |
                    UnicodeBlock::HangulJamoExtendedB |
                    UnicodeBlock::HangulCompatibilityJamo => {
                        families.push("Malgun Gothic");
                    },

                    UnicodeBlock::Ethiopic |
                    UnicodeBlock::EthiopicSupplement |
                    UnicodeBlock::EthiopicExtended |
                    UnicodeBlock::EthiopicExtendedA => {
                        families.push("Nyala");
                    },

                    UnicodeBlock::Cherokee => {
                        families.push("Plantagenet Cherokee");
                    },

                    UnicodeBlock::UnifiedCanadianAboriginalSyllabics |
                    UnicodeBlock::UnifiedCanadianAboriginalSyllabicsExtended => {
                        families.push("Euphemia");
                        families.push("Segoe UI");
                    },

                    UnicodeBlock::Khmer | UnicodeBlock::KhmerSymbols => {
                        families.push("Khmer UI");
                        families.push("Leelawadee UI");
                    },

                    UnicodeBlock::Mongolian => {
                        families.push("Mongolian Baiti");
                    },

                    UnicodeBlock::TaiLe => {
                        families.push("Microsoft Tai Le");
                    },

                    UnicodeBlock::NewTaiLue => {
                        families.push("Microsoft New Tai Lue");
                    },

                    UnicodeBlock::Buginese |
                    UnicodeBlock::TaiTham |
                    UnicodeBlock::CombiningDiacriticalMarksExtended => {
                        families.push("Leelawadee UI");
                    },

                    UnicodeBlock::GeneralPunctuation |
                    UnicodeBlock::SuperscriptsandSubscripts |
                    UnicodeBlock::CurrencySymbols |
                    UnicodeBlock::CombiningDiacriticalMarksforSymbols |
                    UnicodeBlock::LetterlikeSymbols |
                    UnicodeBlock::NumberForms |
                    UnicodeBlock::Arrows |
                    UnicodeBlock::MathematicalOperators |
                    UnicodeBlock::MiscellaneousTechnical |
                    UnicodeBlock::ControlPictures |
                    UnicodeBlock::OpticalCharacterRecognition |
                    UnicodeBlock::EnclosedAlphanumerics |
                    UnicodeBlock::BoxDrawing |
                    UnicodeBlock::BlockElements |
                    UnicodeBlock::GeometricShapes |
                    UnicodeBlock::MiscellaneousSymbols |
                    UnicodeBlock::Dingbats |
                    UnicodeBlock::MiscellaneousMathematicalSymbolsA |
                    UnicodeBlock::SupplementalArrowsA |
                    UnicodeBlock::SupplementalArrowsB |
                    UnicodeBlock::MiscellaneousMathematicalSymbolsB |
                    UnicodeBlock::SupplementalMathematicalOperators |
                    UnicodeBlock::MiscellaneousSymbolsandArrows |
                    UnicodeBlock::Glagolitic |
                    UnicodeBlock::LatinExtendedC |
                    UnicodeBlock::Coptic => {
                        families.push("Segoe UI");
                        families.push("Segoe UI Symbol");
                        families.push("Cambria");
                        families.push("Meiryo");
                        families.push("Lucida Sans Unicode");
                        families.push("Ebrima");
                    },

                    UnicodeBlock::GeorgianSupplement |
                    UnicodeBlock::Tifinagh |
                    UnicodeBlock::CyrillicExtendedA |
                    UnicodeBlock::SupplementalPunctuation |
                    UnicodeBlock::CJKRadicalsSupplement |
                    UnicodeBlock::KangxiRadicals |
                    UnicodeBlock::IdeographicDescriptionCharacters => {
                        families.push("Segoe UI");
                        families.push("Segoe UI Symbol");
                        families.push("Meiryo");
                    },

                    UnicodeBlock::BraillePatterns => {
                        families.push("Segoe UI Symbol");
                    },

                    UnicodeBlock::CJKSymbolsandPunctuation |
                    UnicodeBlock::Hiragana |
                    UnicodeBlock::Katakana |
                    UnicodeBlock::Bopomofo |
                    UnicodeBlock::Kanbun |
                    UnicodeBlock::BopomofoExtended |
                    UnicodeBlock::CJKStrokes |
                    UnicodeBlock::KatakanaPhoneticExtensions |
                    UnicodeBlock::CJKUnifiedIdeographs => {
                        // Simplified Chinese fonts cover these blocks most completely.
                        // Yu Gothic is a Japanese font that omits many PRC-specific
                        // ideographs, so keep it after the Chinese options.
                        families.push("Microsoft YaHei");
                        families.push("Microsoft YaHei UI");
                        families.push("SimSun");
                        families.push("SimHei");
                        families.push("Microsoft JhengHei");
                        families.push("Yu Gothic");
                        families.push("Meiryo");
                    },

                    UnicodeBlock::EnclosedCJKLettersandMonths => {
                        families.push("Malgun Gothic");
                    },

                    UnicodeBlock::YijingHexagramSymbols => {
                        families.push("Segoe UI Symbol");
                    },

                    UnicodeBlock::YiSyllables | UnicodeBlock::YiRadicals => {
                        families.push("Microsoft Yi Baiti");
                        families.push("Segoe UI");
                    },

                    UnicodeBlock::Vai |
                    UnicodeBlock::CyrillicExtendedB |
                    UnicodeBlock::Bamum |
                    UnicodeBlock::ModifierToneLetters |
                    UnicodeBlock::LatinExtendedD => {
                        families.push("Ebrima");
                        families.push("Segoe UI");
                        families.push("Cambria Math");
                    },

                    UnicodeBlock::SylotiNagri |
                    UnicodeBlock::CommonIndicNumberForms |
                    UnicodeBlock::Phagspa |
                    UnicodeBlock::Saurashtra |
                    UnicodeBlock::DevanagariExtended => {
                        families.push("Microsoft PhagsPa");
                        families.push("Nirmala UI");
                    },

                    UnicodeBlock::KayahLi | UnicodeBlock::Rejang | UnicodeBlock::Javanese => {
                        families.push("Malgun Gothic");
                        families.push("Javanese Text");
                        families.push("Leelawadee UI");
                    },

                    UnicodeBlock::AlphabeticPresentationForms => {
                        families.push("Microsoft Uighur");
                        families.push("Gabriola");
                        families.push("Sylfaen");
                    },

                    UnicodeBlock::ArabicPresentationFormsA |
                    UnicodeBlock::ArabicPresentationFormsB => {
                        families.push("Traditional Arabic");
                        families.push("Arabic Typesetting");
                    },

                    UnicodeBlock::VariationSelectors |
                    UnicodeBlock::VerticalForms |
                    UnicodeBlock::CombiningHalfMarks |
                    UnicodeBlock::CJKCompatibilityForms |
                    UnicodeBlock::SmallFormVariants |
                    UnicodeBlock::HalfwidthandFullwidthForms |
                    UnicodeBlock::Specials => {
                        families.push("Microsoft JhengHei");
                    },

                    _ => {},
                }
            }
        },

        // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Multilingual_Plane
        1 => {
            families.push("Segoe UI Symbol");
            families.push("Ebrima");
            families.push("Nirmala UI");
            families.push("Cambria Math");
        },

        _ => {},
    }

    families.push("Arial Unicode MS");
    families
}

fn font_template_descriptor_from_font(font: &Font) -> FontTemplateDescriptor {
    let style = match font.style() {
        FontStyle::Normal => StyleFontStyle::NORMAL,
        FontStyle::Oblique => StyleFontStyle::OBLIQUE,
        FontStyle::Italic => StyleFontStyle::ITALIC,
    };
    let weight = StyleFontWeight::from_float(font.weight().to_u32() as f32);
    let stretch = match font.stretch() {
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
    .compute();
    FontTemplateDescriptor::new(weight, stretch, style)
}

pub(crate) fn default_system_generic_font_family(
    generic: GenericFontFamily,
) -> LowercaseFontFamilyName {
    match generic {
        GenericFontFamily::None | GenericFontFamily::Serif => "Times New Roman",
        GenericFontFamily::SansSerif => "Arial",
        GenericFontFamily::Monospace => "Courier New",
        GenericFontFamily::Cursive => "Comic Sans MS",
        GenericFontFamily::Fantasy => "Impact",
        GenericFontFamily::SystemUi => "Segoe UI",
    }
    .into()
}
