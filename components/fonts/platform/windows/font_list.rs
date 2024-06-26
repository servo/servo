/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::hash::Hash;
use std::sync::Arc;

use base::text::{unicode_plane, UnicodeBlock, UnicodeBlockMethod};
use dwrote::{Font, FontCollection, FontDescriptor, FontStretch, FontStyle};
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use style::values::computed::font::GenericFontFamily;
use style::values::computed::{FontStyle as StyleFontStyle, FontWeight as StyleFontWeight};
use style::values::specified::font::FontStretchKeyword;

use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontTemplate,
    FontTemplateDescriptor, LowercaseFontFamilyName,
};

pub fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    let system_fc = FontCollection::system();
    for family in system_fc.families_iter() {
        callback(family.name());
    }
}

/// An identifier for a local font on a Windows system.
#[derive(Clone, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    /// The FontDescriptor of this font.
    #[ignore_malloc_size_of = "dwrote does not support MallocSizeOf"]
    pub font_descriptor: Arc<FontDescriptor>,
}

impl LocalFontIdentifier {
    pub fn index(&self) -> u32 {
        FontCollection::system()
            .get_font_from_descriptor(&self.font_descriptor)
            .map_or(0, |font| font.create_font_face().get_index())
    }

    pub(crate) fn read_data_from_file(&self) -> Vec<u8> {
        let font = FontCollection::system()
            .get_font_from_descriptor(&self.font_descriptor)
            .unwrap();
        let face = font.create_font_face();
        let files = face.get_files();
        assert!(!files.is_empty());
        files[0].get_font_file_bytes()
    }
}

impl Eq for LocalFontIdentifier {}

impl Hash for LocalFontIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.font_descriptor.family_name.hash(state);
        self.font_descriptor.weight.to_u32().hash(state);
        self.font_descriptor.stretch.to_u32().hash(state);
        self.font_descriptor.style.to_u32().hash(state);
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
{
    let system_fc = FontCollection::system();
    if let Some(family) = system_fc.get_font_family_by_name(family_name) {
        let count = family.get_font_count();
        for i in 0..count {
            let font = family.get_font(i);
            let template_descriptor = (&font).into();
            let local_font_identifier = LocalFontIdentifier {
                font_descriptor: Arc::new(font.to_descriptor()),
            };
            callback(FontTemplate::new_for_local_font(
                local_font_identifier,
                template_descriptor,
            ))
        }
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
                        families.push("Microsoft YaHei");
                        families.push("Yu Gothic");
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

impl From<&Font> for FontTemplateDescriptor {
    fn from(font: &Font) -> Self {
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
}

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
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
