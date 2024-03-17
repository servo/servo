/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use log::debug;
use serde::{Deserialize, Serialize};
use style::Atom;
use ucd::{Codepoint, UnicodeBlock};

use crate::text::util::unicode_plane;

/// An identifier for a local font on a MacOS system. These values comes from the CoreText
/// CTFontCollection. Note that `path` here is required. We do not load fonts that do not
/// have paths.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    pub postscript_name: Atom,
    pub path: Atom,
}

pub fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    let family_names = core_text::font_collection::get_family_names();
    for family_name in family_names.iter() {
        callback(family_name.to_string());
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(LocalFontIdentifier),
{
    debug!("Looking for faces of family: {}", family_name);

    let family_collection = core_text::font_collection::create_for_family(family_name);
    if let Some(family_collection) = family_collection {
        if let Some(family_descriptors) = family_collection.get_descriptors() {
            for family_descriptor in family_descriptors.iter() {
                let path = family_descriptor.font_path();
                let path = match path.as_ref().and_then(|path| path.to_str()) {
                    Some(path) => path,
                    None => continue,
                };
                callback(LocalFontIdentifier {
                    postscript_name: Atom::from(family_descriptor.font_name()),
                    path: Atom::from(path),
                })
            }
        }
    }
}

pub fn system_default_family(_generic_name: &str) -> Option<String> {
    None
}

// Based on gfxPlatformMac::GetCommonFallbackFonts() in Gecko
pub fn fallback_font_families(codepoint: Option<char>) -> Vec<&'static str> {
    let mut families = vec!["Lucida Grande"];

    if let Some(codepoint) = codepoint {
        match unicode_plane(codepoint) {
            // https://en.wikipedia.org/wiki/Plane_(Unicode)#Basic_Multilingual_Plane
            0 => {
                if let Some(block) = codepoint.block() {
                    match block {
                        UnicodeBlock::Arabic |
                        UnicodeBlock::Syriac |
                        UnicodeBlock::ArabicSupplement |
                        UnicodeBlock::Thaana |
                        UnicodeBlock::NKo => {
                            families.push("Geeza Pro");
                        },

                        UnicodeBlock::Devanagari => {
                            families.push("Devanagari Sangam MN");
                        },

                        UnicodeBlock::Gurmukhi => {
                            families.push("Gurmukhi MN");
                        },

                        UnicodeBlock::Gujarati => {
                            families.push("Gujarati Sangam MN");
                        },

                        UnicodeBlock::Tamil => {
                            families.push("Tamil MN");
                        },

                        UnicodeBlock::Lao => {
                            families.push("Lao MN");
                        },

                        UnicodeBlock::Tibetan => {
                            families.push("Songti SC");
                        },

                        UnicodeBlock::Myanmar => {
                            families.push("Myanmar MN");
                        },

                        UnicodeBlock::Ethiopic |
                        UnicodeBlock::EthiopicSupplement |
                        UnicodeBlock::EthiopicExtended |
                        UnicodeBlock::EthiopicExtendedA => {
                            families.push("Kefa");
                        },

                        UnicodeBlock::Cherokee => {
                            families.push("Plantagenet Cherokee");
                        },

                        UnicodeBlock::UnifiedCanadianAboriginalSyllabics |
                        UnicodeBlock::UnifiedCanadianAboriginalSyllabicsExtended => {
                            families.push("Euphemia UCAS");
                        },

                        UnicodeBlock::Mongolian |
                        UnicodeBlock::YiSyllables |
                        UnicodeBlock::YiRadicals => {
                            families.push("STHeiti");
                        },

                        UnicodeBlock::Khmer | UnicodeBlock::KhmerSymbols => {
                            families.push("Khmer MN");
                        },

                        UnicodeBlock::TaiLe => {
                            families.push("Microsoft Tai Le");
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
                        UnicodeBlock::SupplementalPunctuation => {
                            families.push("Hiragino Kaku Gothic ProN");
                            families.push("Apple Symbols");
                            families.push("Menlo");
                            families.push("STIXGeneral");
                        },

                        UnicodeBlock::BraillePatterns => {
                            families.push("Apple Braille");
                        },

                        UnicodeBlock::Bopomofo |
                        UnicodeBlock::HangulCompatibilityJamo |
                        UnicodeBlock::Kanbun |
                        UnicodeBlock::BopomofoExtended |
                        UnicodeBlock::CJKStrokes |
                        UnicodeBlock::KatakanaPhoneticExtensions => {
                            families.push("Hiragino Sans GB");
                        },

                        UnicodeBlock::YijingHexagramSymbols |
                        UnicodeBlock::CyrillicExtendedB |
                        UnicodeBlock::Bamum |
                        UnicodeBlock::ModifierToneLetters |
                        UnicodeBlock::LatinExtendedD |
                        UnicodeBlock::ArabicPresentationFormsA |
                        UnicodeBlock::HalfwidthandFullwidthForms |
                        UnicodeBlock::Specials => {
                            families.push("Apple Symbols");
                        },

                        _ => {},
                    }
                }
            },

            // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Multilingual_Plane
            1 => {
                families.push("Apple Symbols");
                families.push("STIXGeneral");
            },

            // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Ideographic_Plane
            2 => {
                // Systems with MS Office may have these fonts
                families.push("MingLiU-ExtB");
                families.push("SimSun-ExtB");
            },

            _ => {},
        }
    }

    families.push("Geneva");
    families.push("Arial Unicode MS");
    families
}

pub static SANS_SERIF_FONT_FAMILY: &str = "Helvetica";
