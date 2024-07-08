/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fs::File;
use std::io::Read;
use std::path::Path;

use base::text::{unicode_plane, UnicodeBlock, UnicodeBlockMethod};
use log::debug;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use style::values::computed::font::GenericFontFamily;
use style::Atom;
use unicode_script::Script;
use webrender_api::NativeFontHandle;

use crate::platform::add_noto_fallback_families;
use crate::platform::font::CoreTextFontTraitsMapping;
use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontTemplate,
    FontTemplateDescriptor, LowercaseFontFamilyName,
};

/// An identifier for a local font on a MacOS system. These values comes from the CoreText
/// CTFontCollection. Note that `path` here is required. We do not load fonts that do not
/// have paths.
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
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
                callback(FontTemplate::new_for_local_font(identifier, descriptor));
            }
        }
    }
}

/// Get the list of fallback fonts given an optional codepoint. This is
/// based on `gfxPlatformMac::GetCommonFallbackFonts()` in Gecko from
/// <https://searchfox.org/mozilla-central/source/gfx/thebes/gfxPlatformMac.cpp>.
pub fn fallback_font_families(options: FallbackFontSelectionOptions) -> Vec<&'static str> {
    let mut families = Vec::new();
    if options.presentation_preference == EmojiPresentationPreference::Emoji {
        families.push("Apple Color Emoji");
    }

    let script = Script::from(options.character);
    if let Some(block) = options.character.block() {
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
                if options.character as u32 > 0x10000 {
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
            UnicodeBlock::Deseret => families.push("Baskerville"),
            UnicodeBlock::Devanagari | UnicodeBlock::DevanagariExtended => {
                families.push("Devanagari Sangam MN")
            },
            UnicodeBlock::Ethiopic |
            UnicodeBlock::EthiopicExtended |
            UnicodeBlock::EthiopicExtendedA |
            UnicodeBlock::EthiopicSupplement => families.push("Kefa"),
            UnicodeBlock::Georgian | UnicodeBlock::GeorgianSupplement => families.push("Helvetica"),
            UnicodeBlock::Gujarati => families.push("Gujarati Sangam MN"),
            UnicodeBlock::Gurmukhi => families.push("Gurmukhi MN"),
            UnicodeBlock::Hebrew => families.push("Lucida Grande"),
            UnicodeBlock::Kannada => families.push("Kannada MN"),
            UnicodeBlock::Khmer => families.push("Khmer MN"),
            UnicodeBlock::Lao => families.push("Lao MN"),
            UnicodeBlock::Malayalam => families.push("Malayalam Sangam MN"),
            UnicodeBlock::Myanmar |
            UnicodeBlock::MyanmarExtendedA |
            UnicodeBlock::MyanmarExtendedB => families.push("Myanmar MN"),
            UnicodeBlock::Oriya => families.push("Oriya Sangam MN"),
            UnicodeBlock::Sinhala | UnicodeBlock::SinhalaArchaicNumbers => {
                families.push("Sinhala Sangam MN")
            },
            UnicodeBlock::Tamil => families.push("Tamil MN"),
            UnicodeBlock::Telugu => families.push("Telugu MN"),
            UnicodeBlock::Thaana => {
                families.push("Thonburi");
            },
            UnicodeBlock::Tibetan => families.push("Kailasa"),
            UnicodeBlock::UnifiedCanadianAboriginalSyllabics |
            UnicodeBlock::UnifiedCanadianAboriginalSyllabicsExtended => {
                families.push("Euphemia UCAS")
            },
            UnicodeBlock::YiSyllables | UnicodeBlock::YiRadicals => {
                families.push("STHeiti");
            },
            UnicodeBlock::BraillePatterns => families.push("Apple Braille"),
            _ => {},
        }
    }

    add_noto_fallback_families(options, &mut families);

    // https://en.wikipedia.org/wiki/Plane_(Unicode)#Supplementary_Multilingual_Plane
    let unicode_plane = unicode_plane(options.character);
    if let 1 = unicode_plane {
        let b = (options.character as u32) >> 8;
        if b == 0x27 {
            families.push("Zapf Dingbats");
        }
        families.push("Geneva");
        families.push("Apple Symbols");
        families.push("STIXGeneral");
        families.push("Hiragino Sans");
        families.push("Hiragino Kaku Gothic ProN");
    }

    families.push("Arial Unicode MS");
    families
}

pub fn default_system_generic_font_family(generic: GenericFontFamily) -> LowercaseFontFamilyName {
    match generic {
        GenericFontFamily::None | GenericFontFamily::Serif => "Times",
        GenericFontFamily::SansSerif => "Helvetica",
        GenericFontFamily::Monospace => "Menlo",
        GenericFontFamily::Cursive => "Apple Chancery",
        GenericFontFamily::Fantasy => "Papyrus",
        GenericFontFamily::SystemUi => "Menlo",
    }
    .into()
}
