/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;
use std::path::Path;

use log::debug;
use serde::{Deserialize, Serialize};
use style::Atom;
use ucd::{Codepoint, UnicodeBlock};
use unicode_script::Script;
use webrender_api::NativeFontHandle;

use crate::font_template::{FontTemplate, FontTemplateDescriptor};
use crate::platform::font::CoreTextFontTraitsMapping;
use crate::text::util::unicode_plane;

/// An identifier for a local font on a MacOS system. These values comes from the CoreText
/// CTFontCollection. Note that `path` here is required. We do not load fonts that do not
/// have paths.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LocalFontIdentifier {
    pub postscript_name: Atom,
    pub path: Atom,
}

impl LocalFontIdentifier {
    pub(crate) fn native_font_handle(&self) -> NativeFontHandle {
        NativeFontHandle {
            name: self.postscript_name.to_string(),
            path: self.path.to_string(),
        }
    }

    pub(crate) fn index(&self) -> u32 {
        0
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
    let family_names = core_text::font_collection::get_family_names();
    for family_name in family_names.iter() {
        callback(family_name.to_string());
    }
}

pub fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
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

                let traits = family_descriptor.traits();
                let descriptor =
                    FontTemplateDescriptor::new(traits.weight(), traits.stretch(), traits.style());
                let identifier = LocalFontIdentifier {
                    postscript_name: Atom::from(family_descriptor.font_name()),
                    path: Atom::from(path),
                };
                callback(FontTemplate::new_local(identifier, descriptor));
            }
        }
    }
}

pub fn system_default_family(_generic_name: &str) -> Option<String> {
    None
}

/// Get the list of fallback fonts given an optional codepoint. This is
/// based on `gfxPlatformMac::GetCommonFallbackFonts()` in Gecko from
/// <https://searchfox.org/mozilla-central/source/gfx/thebes/gfxPlatformMac.cpp>.
pub fn fallback_font_families(codepoint: Option<char>) -> Vec<&'static str> {
    let mut families = vec!["Lucida Grande"];
    let Some(codepoint) = codepoint else {
        families.push("Geneva");
        families.push("Arial Unicode MS");
        return families;
    };

    let script = Script::from(codepoint);
    if let Some(block) = codepoint.block() {
        match block {
            // In most cases, COMMON and INHERITED characters will be merged into
            // their context, but if they occur without any specific script context
            // we'll just try common default fonts here.
            _ if matches!(
                script,
                Script::Common |
                    Script::Inherited |
                    Script::Latin |
                    Script::Cyrillic |
                    Script::Greek
            ) =>
            {
                families.push("Lucida Grande");
            },
            // CJK-related script codes are a bit troublesome because of unification;
            // we'll probably just get HAN much of the time, so the choice of which
            // language font to try for fallback is rather arbitrary. Usually, though,
            // we hope that font prefs will have handled this earlier.
            _ if matches!(script, Script::Bopomofo | Script::Han) => {
                // TODO: Need to differentiate between traditional and simplified Han here!
                families.push("Songti SC");
                if codepoint as u32 > 0x10000 {
                    // macOS installations with MS Office may have these -ExtB fonts
                    families.push("SimSun-ExtB");
                }
            },
            UnicodeBlock::Hiragana |
            UnicodeBlock::Katakana |
            UnicodeBlock::KatakanaPhoneticExtensions => {
                families.push("Hiragino Sans");
                families.push("Hiragino Kaku Gothic ProN");
            },
            UnicodeBlock::HangulJamo |
            UnicodeBlock::HangulJamoExtendedA |
            UnicodeBlock::HangulJamoExtendedB |
            UnicodeBlock::HangulCompatibilityJamo |
            UnicodeBlock::HangulSyllables => {
                families.push("Nanum Gothic");
                families.push("Apple SD Gothic Neo");
            },
            UnicodeBlock::Arabic => families.push("Geeza Pro"),
            UnicodeBlock::Armenian => families.push("Mshtakan"),
            UnicodeBlock::Bengali => families.push("Bangla Sangam MN"),
            UnicodeBlock::Cherokee => families.push("Plantagenet Cherokee"),
            UnicodeBlock::Coptic => families.push("Noto Sans Coptic"),
            UnicodeBlock::Deseret => families.push("Baskerville"),
            UnicodeBlock::Devanagari | UnicodeBlock::DevanagariExtended => {
                families.push("Devanagari Sangam MN")
            },
            UnicodeBlock::Ethiopic |
            UnicodeBlock::EthiopicExtended |
            UnicodeBlock::EthiopicExtendedA |
            UnicodeBlock::EthiopicSupplement => families.push("Kefa"),
            UnicodeBlock::Georgian | UnicodeBlock::GeorgianSupplement => families.push("Helvetica"),
            UnicodeBlock::Gothic => families.push("Noto Sans Gothic"),
            UnicodeBlock::Gujarati => families.push("Gujarati Sangam MN"),
            UnicodeBlock::Gurmukhi => families.push("Gurmukhi MN"),
            UnicodeBlock::Hebrew => families.push("Lucida Grande"),
            UnicodeBlock::Kannada => families.push("Kannada MN"),
            UnicodeBlock::Khmer => families.push("Khmer MN"),
            UnicodeBlock::Lao => families.push("Lao MN"),
            UnicodeBlock::Malayalam => families.push("Malayalam Sangam MN"),
            UnicodeBlock::Mongolian | UnicodeBlock::MongolianSupplement => {
                families.push("Noto Sans Mongolian")
            },
            UnicodeBlock::Myanmar |
            UnicodeBlock::MyanmarExtendedA |
            UnicodeBlock::MyanmarExtendedB => families.push("Myanmar MN"),
            UnicodeBlock::Ogham => families.push("Noto Sans Ogham"),
            UnicodeBlock::OldItalic => families.push("Noto Sans Old Italic"),
            UnicodeBlock::Oriya => families.push("Oriya Sangam MN"),
            UnicodeBlock::Runic => families.push("Noto Sans Runic"),
            UnicodeBlock::Sinhala | UnicodeBlock::SinhalaArchaicNumbers => {
                families.push("Sinhala Sangam MN")
            },
            UnicodeBlock::Syriac => families.push("Noto Sans Syriac"),
            UnicodeBlock::Tamil => families.push("Tamil MN"),
            UnicodeBlock::Telugu => families.push("Telugu MN"),
            UnicodeBlock::Thaana => {
                families.push("Noto Sans Thaana");
                families.push("Thonburi");
            },
            UnicodeBlock::Tibetan => families.push("Kailasa"),
            UnicodeBlock::UnifiedCanadianAboriginalSyllabics |
            UnicodeBlock::UnifiedCanadianAboriginalSyllabicsExtended => {
                families.push("Euphemia UCAS")
            },
            UnicodeBlock::YiSyllables | UnicodeBlock::YiRadicals => {
                families.push("Noto Sans Yi");
                families.push("STHeiti");
            },
            UnicodeBlock::Tagalog => families.push("Noto Sans Tagalog"),
            UnicodeBlock::Hanunoo => families.push("Noto Sans Hanunoo"),
            UnicodeBlock::Buhid => families.push("Noto Sans Buhid"),
            UnicodeBlock::Tagbanwa => families.push("Noto Sans Tagbanwa"),
            UnicodeBlock::BraillePatterns => families.push("Apple Braille"),
            UnicodeBlock::CypriotSyllabary => families.push("Noto Sans Cypriot"),
            UnicodeBlock::Limbu => families.push("Noto Sans Limbu"),
            UnicodeBlock::LinearBIdeograms | UnicodeBlock::LinearBSyllabary => {
                families.push("Noto Sans Linear B")
            },
            UnicodeBlock::Osmanya => families.push("Noto Sans Osmanya"),
            UnicodeBlock::Shavian => families.push("Noto Sans Shavian"),
            UnicodeBlock::TaiLe => families.push("Noto Sans Tai Le"),
            UnicodeBlock::Ugaritic => families.push("Noto Sans Ugaritic"),
            UnicodeBlock::Buginese => families.push("Noto Sans Buginese"),
            UnicodeBlock::Glagolitic | UnicodeBlock::GlagoliticSupplement => {
                families.push("Noto Sans Glagolitic")
            },
            UnicodeBlock::Kharoshthi => families.push("Noto Sans Kharoshthi"),
            UnicodeBlock::SylotiNagri => families.push("Noto Sans Syloti Nagri"),
            UnicodeBlock::NewTaiLue => families.push("Noto Sans New Tai Lue"),
            UnicodeBlock::Tifinagh => families.push("Noto Sans Tifinagh"),
            UnicodeBlock::OldPersian => families.push("Noto Sans Old Persian"),
            UnicodeBlock::Balinese => families.push("Noto Sans Balinese"),
            UnicodeBlock::Batak => families.push("Noto Sans Batak"),
            UnicodeBlock::Brahmi => families.push("Noto Sans Brahmi"),
            UnicodeBlock::Cham => families.push("Noto Sans Cham"),
            UnicodeBlock::EgyptianHieroglyphs => families.push("Noto Sans Egyptian Hieroglyphs"),
            UnicodeBlock::PahawhHmong => families.push("Noto Sans Pahawh Hmong"),
            UnicodeBlock::OldHungarian => families.push("Noto Sans Old Hungarian"),
            UnicodeBlock::Javanese => families.push("Noto Sans Javanese"),
            UnicodeBlock::KayahLi => families.push("Noto Sans Kayah Li"),
            UnicodeBlock::Lepcha => families.push("Noto Sans Lepcha"),
            UnicodeBlock::LinearA => families.push("Noto Sans Linear A"),
            UnicodeBlock::Mandaic => families.push("Noto Sans Mandaic"),
            UnicodeBlock::NKo => families.push("Noto Sans NKo"),
            UnicodeBlock::OldTurkic => families.push("Noto Sans Old Turkic"),
            UnicodeBlock::OldPermic => families.push("Noto Sans Old Permic"),
            UnicodeBlock::Phagspa => families.push("Noto Sans PhagsPa"),
            UnicodeBlock::Phoenician => families.push("Noto Sans Phoenician"),
            UnicodeBlock::Miao => families.push("Noto Sans Miao"),
            UnicodeBlock::Vai => families.push("Noto Sans Vai"),
            UnicodeBlock::Cuneiform | UnicodeBlock::CuneiformNumbersandPunctuation => {
                families.push("Noto Sans Cuneiform")
            },
            UnicodeBlock::Carian => families.push("Noto Sans Carian"),
            UnicodeBlock::TaiTham => families.push("Noto Sans Tai Tham"),
            UnicodeBlock::Lycian => families.push("Noto Sans Lycian"),
            UnicodeBlock::Lydian => families.push("Noto Sans Lydian"),
            UnicodeBlock::OlChiki => families.push("Noto Sans Ol Chiki"),
            UnicodeBlock::Rejang => families.push("Noto Sans Rejang"),
            UnicodeBlock::Saurashtra => families.push("Noto Sans Saurashtra"),
            UnicodeBlock::Sundanese => families.push("Noto Sans Sundanese"),
            UnicodeBlock::MeeteiMayek | UnicodeBlock::MeeteiMayekExtensions => {
                families.push("Noto Sans Meetei Mayek")
            },
            UnicodeBlock::ImperialAramaic => families.push("Noto Sans Imperial Aramaic"),
            UnicodeBlock::Avestan => families.push("Noto Sans Avestan"),
            UnicodeBlock::Chakma => families.push("Noto Sans Chakma"),
            UnicodeBlock::Kaithi => families.push("Noto Sans Kaithi"),
            UnicodeBlock::Manichaean => families.push("Noto Sans Manichaean"),
            UnicodeBlock::InscriptionalPahlavi => families.push("Noto Sans Inscriptional Pahlavi"),
            UnicodeBlock::PsalterPahlavi => families.push("Noto Sans Psalter Pahlavi"),
            UnicodeBlock::InscriptionalParthian => {
                families.push("Noto Sans Inscriptional Parthian")
            },
            UnicodeBlock::Samaritan => families.push("Noto Sans Samaritan"),
            UnicodeBlock::TaiViet => families.push("Noto Sans Tai Viet"),
            UnicodeBlock::Bamum | UnicodeBlock::BamumSupplement => families.push("Noto Sans Bamum"),
            UnicodeBlock::Lisu => families.push("Noto Sans Lisu"),
            UnicodeBlock::OldSouthArabian => families.push("Noto Sans Old South Arabian"),
            UnicodeBlock::BassaVah => families.push("Noto Sans Bassa Vah"),
            UnicodeBlock::Duployan => families.push("Noto Sans Duployan"),
            UnicodeBlock::Elbasan => families.push("Noto Sans Elbasan"),
            UnicodeBlock::Grantha => families.push("Noto Sans Grantha"),
            UnicodeBlock::MendeKikakui => families.push("Noto Sans Mende Kikakui"),
            UnicodeBlock::MeroiticCursive | UnicodeBlock::MeroiticHieroglyphs => {
                families.push("Noto Sans Meroitic")
            },
            UnicodeBlock::OldNorthArabian => families.push("Noto Sans Old North Arabian"),
            UnicodeBlock::Nabataean => families.push("Noto Sans Nabataean"),
            UnicodeBlock::Palmyrene => families.push("Noto Sans Palmyrene"),
            UnicodeBlock::Khudawadi => families.push("Noto Sans Khudawadi"),
            UnicodeBlock::WarangCiti => families.push("Noto Sans Warang Citi"),
            UnicodeBlock::Mro => families.push("Noto Sans Mro"),
            UnicodeBlock::Sharada => families.push("Noto Sans Sharada"),
            UnicodeBlock::SoraSompeng => families.push("Noto Sans Sora Sompeng"),
            UnicodeBlock::Takri => families.push("Noto Sans Takri"),
            UnicodeBlock::Khojki => families.push("Noto Sans Khojki"),
            UnicodeBlock::Tirhuta => families.push("Noto Sans Tirhuta"),
            UnicodeBlock::CaucasianAlbanian => families.push("Noto Sans Caucasian Albanian"),
            UnicodeBlock::Mahajani => families.push("Noto Sans Mahajani"),
            UnicodeBlock::Ahom => families.push("Noto Serif Ahom"),
            UnicodeBlock::Hatran => families.push("Noto Sans Hatran"),
            UnicodeBlock::Modi => families.push("Noto Sans Modi"),
            UnicodeBlock::Multani => families.push("Noto Sans Multani"),
            UnicodeBlock::PauCinHau => families.push("Noto Sans Pau Cin Hau"),
            UnicodeBlock::Siddham => families.push("Noto Sans Siddham"),
            UnicodeBlock::Adlam => families.push("Noto Sans Adlam"),
            UnicodeBlock::Bhaiksuki => families.push("Noto Sans Bhaiksuki"),
            UnicodeBlock::Marchen => families.push("Noto Sans Marchen"),
            UnicodeBlock::Newa => families.push("Noto Sans Newa"),
            UnicodeBlock::Osage => families.push("Noto Sans Osage"),
            _ if script == Script::Hanifi_Rohingya => families.push("Noto Sans Hanifi Rohingya"),
            _ if script == Script::Wancho => families.push("Noto Sans Wancho"),
            _ => {},
        }
    }

    // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Multilingual_Plane
    let unicode_plane = unicode_plane(codepoint);
    if let 1 = unicode_plane {
        let b = (codepoint as u32) >> 8;
        if b >= 0x1f0 && b < 0x1f7 {
            families.push("Apple Color Emoji");
        }
        families.push("Apple Symbols");
        families.push("STIXGeneral");
    }

    families.push("Geneva");
    families.push("Arial Unicode MS");
    families
}

pub static SANS_SERIF_FONT_FAMILY: &str = "Helvetica";
