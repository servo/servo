/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dwrote::{Font, FontDescriptor, FontCollection};
use servo_atoms::Atom;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{Ordering, AtomicUsize};
use text::util::unicode_plane;
use ucd::{Codepoint, UnicodeBlock};

lazy_static! {
    static ref FONT_ATOM_COUNTER: AtomicUsize = AtomicUsize::new(1);
    static ref FONT_ATOM_MAP: Mutex<HashMap<Atom, FontDescriptor>> = Mutex::new(HashMap::new());
}

pub static SANS_SERIF_FONT_FAMILY: &'static str = "Arial";

pub fn system_default_family(_: &str) -> Option<String> {
    Some("Verdana".to_owned())
}

pub fn for_each_available_family<F>(mut callback: F) where F: FnMut(String) {
    let system_fc = FontCollection::system();
    for family in system_fc.families_iter() {
        callback(family.name());
    }
}

// for_each_variation is supposed to return a string that can be
// atomized and then uniquely used to return back to this font.
// Some platforms use the full postscript name (MacOS X), or
// a font filename.
//
// For windows we're going to use just a basic integer value that
// we'll stringify, and then put them all in a HashMap with
// the actual FontDescriptor there.

pub fn for_each_variation<F>(family_name: &str, mut callback: F) where F: FnMut(String) {
    let system_fc = FontCollection::system();
    if let Some(family) = system_fc.get_font_family_by_name(family_name) {
        let count = family.get_font_count();
        for i in 0..count {
            let font = family.get_font(i);
            let index = FONT_ATOM_COUNTER.fetch_add(1, Ordering::Relaxed);
            let index_str = format!("{}", index);
            let atom = Atom::from(index_str.clone());

            {
                let descriptor = font.to_descriptor();
                let mut fonts = FONT_ATOM_MAP.lock().unwrap();
                fonts.insert(atom, descriptor);
            }

            callback(index_str);
        }
    }
}

pub fn descriptor_from_atom(ident: &Atom) -> FontDescriptor {
    let fonts = FONT_ATOM_MAP.lock().unwrap();
    fonts.get(ident).unwrap().clone()
}

pub fn font_from_atom(ident: &Atom) -> Font {
    let fonts = FONT_ATOM_MAP.lock().unwrap();
    FontCollection::system().get_font_from_descriptor(fonts.get(ident).unwrap()).unwrap()
}

// Based on gfxWindowsPlatform::GetCommonFallbackFonts() in Gecko
pub fn fallback_font_families(codepoint: Option<char>) -> Vec<&'static str> {
    let mut families = vec!("Arial");

    if let Some(codepoint) = codepoint {
        match unicode_plane(codepoint) {
            // https://en.wikipedia.org/wiki/Plane_(Unicode)#Basic_Multilingual_Plane
            0 => {
                if let Some(block) = codepoint.block() {
                    match block {
                        UnicodeBlock::CyrillicSupplement |
                        UnicodeBlock::Armenian |
                        UnicodeBlock::Hebrew => {
                            families.push("Estrangelo Edessa");
                            families.push("Cambria");
                        }

                        UnicodeBlock::Arabic |
                        UnicodeBlock::ArabicSupplement => {
                            families.push("Microsoft Uighur");
                        }

                        UnicodeBlock::Syriac => {
                            families.push("Estrangelo Edessa");
                        }

                        UnicodeBlock::Thaana => {
                            families.push("MV Boli");
                        }

                        UnicodeBlock::NKo => {
                            families.push("Ebrima");
                        }

                        UnicodeBlock::Devanagari |
                        UnicodeBlock::Bengali => {
                            families.push("Nirmala UI");
                            families.push("Utsaah");
                            families.push("Aparajita");
                        }

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
                        }

                        UnicodeBlock::Thai => {
                            families.push("Leelawadee UI");
                        }

                        UnicodeBlock::Lao => {
                            families.push("Lao UI");
                        }

                        UnicodeBlock::Myanmar |
                        UnicodeBlock::MyanmarExtendedA |
                        UnicodeBlock::MyanmarExtendedB => {
                            families.push("Myanmar Text");
                        }

                        UnicodeBlock::HangulJamo |
                        UnicodeBlock::HangulJamoExtendedA |
                        UnicodeBlock::HangulSyllables |
                        UnicodeBlock::HangulJamoExtendedB |
                        UnicodeBlock::HangulCompatibilityJamo => {
                            families.push("Malgun Gothic");
                        }

                        UnicodeBlock::Ethiopic |
                        UnicodeBlock::EthiopicSupplement |
                        UnicodeBlock::EthiopicExtended |
                        UnicodeBlock::EthiopicExtendedA => {
                            families.push("Nyala");
                        }

                        UnicodeBlock::Cherokee => {
                            families.push("Plantagenet Cherokee");
                        }

                        UnicodeBlock::UnifiedCanadianAboriginalSyllabics |
                        UnicodeBlock::UnifiedCanadianAboriginalSyllabicsExtended => {
                            families.push("Euphemia");
                            families.push("Segoe UI");
                        }

                        UnicodeBlock::Khmer |
                        UnicodeBlock::KhmerSymbols => {
                            families.push("Khmer UI");
                            families.push("Leelawadee UI");
                        }

                        UnicodeBlock::Mongolian => {
                            families.push("Mongolian Baiti");
                        }

                        UnicodeBlock::TaiLe => {
                            families.push("Microsoft Tai Le");
                        }

                        UnicodeBlock::NewTaiLue => {
                            families.push("Microsoft New Tai Lue");
                        }

                        UnicodeBlock::Buginese |
                        UnicodeBlock::TaiTham |
                        UnicodeBlock::CombiningDiacriticalMarksExtended => {
                            families.push("Leelawadee UI");
                        }

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
                        }

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
                        }

                        UnicodeBlock::BraillePatterns => {
                            families.push("Segoe UI Symbol");
                        }

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
                        }

                        UnicodeBlock::EnclosedCJKLettersandMonths => {
                            families.push("Malgun Gothic");
                        }

                        UnicodeBlock::YijingHexagramSymbols => {
                            families.push("Segoe UI Symbol");
                        }

                        UnicodeBlock::YiSyllables |
                        UnicodeBlock::YiRadicals => {
                            families.push("Microsoft Yi Baiti");
                            families.push("Segoe UI");
                        }

                        UnicodeBlock::Vai |
                        UnicodeBlock::CyrillicExtendedB |
                        UnicodeBlock::Bamum |
                        UnicodeBlock::ModifierToneLetters |
                        UnicodeBlock::LatinExtendedD => {
                            families.push("Ebrima");
                            families.push("Segoe UI");
                            families.push("Cambria Math");
                        }

                        UnicodeBlock::SylotiNagri |
                        UnicodeBlock::CommonIndicNumberForms |
                        UnicodeBlock::Phagspa |
                        UnicodeBlock::Saurashtra |
                        UnicodeBlock::DevanagariExtended => {
                             families.push("Microsoft PhagsPa");
                             families.push("Nirmala UI");
                        }

                        UnicodeBlock::KayahLi |
                        UnicodeBlock::Rejang |
                        UnicodeBlock::Javanese => {
                             families.push("Malgun Gothic");
                             families.push("Javanese Text");
                             families.push("Leelawadee UI");
                        }

                        UnicodeBlock::AlphabeticPresentationForms => {
                            families.push("Microsoft Uighur");
                            families.push("Gabriola");
                            families.push("Sylfaen");
                        }

                        UnicodeBlock::ArabicPresentationFormsA |
                        UnicodeBlock::ArabicPresentationFormsB => {
                            families.push("Traditional Arabic");
                            families.push("Arabic Typesetting");
                        }

                        UnicodeBlock::VariationSelectors |
                        UnicodeBlock::VerticalForms |
                        UnicodeBlock::CombiningHalfMarks |
                        UnicodeBlock::CJKCompatibilityForms |
                        UnicodeBlock::SmallFormVariants |
                        UnicodeBlock::HalfwidthandFullwidthForms |
                        UnicodeBlock::Specials => {
                            families.push("Microsoft JhengHei");
                        }

                        _ => {}
                    }
                }
            }

            // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Multilingual_Plane
            1 => {
                families.push("Segoe UI Symbol");
                families.push("Ebrima");
                families.push("Nirmala UI");
                families.push("Cambria Math");
            }

            _ => {}
        }
    }

    families.push("Arial Unicode MS");
    families
}
