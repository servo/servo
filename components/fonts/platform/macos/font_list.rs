/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_void;

use base::text::{UnicodeBlock, UnicodeBlockMethod, unicode_plane};
use fonts_traits::LocalFontIdentifier;
use log::debug;
use objc2_core_foundation::{CFDictionary, CFRetained, CFSet, CFString, CFType, CFURL};
use objc2_core_text::{
    CTFontDescriptor, CTFontManagerCopyAvailableFontFamilyNames, kCTFontFamilyNameAttribute,
    kCTFontNameAttribute, kCTFontTraitsAttribute, kCTFontURLAttribute,
};
use style::Atom;
use style::values::computed::font::GenericFontFamily;
use unicode_script::Script;

use crate::platform::add_noto_fallback_families;
use crate::platform::font::font_template_descriptor_from_ctfont_attributes;
use crate::{
    EmojiPresentationPreference, FallbackFontSelectionOptions, FontIdentifier, FontTemplate,
    LowercaseFontFamilyName,
};

pub(crate) fn for_each_available_family<F>(mut callback: F)
where
    F: FnMut(String),
{
    let family_names = unsafe { CTFontManagerCopyAvailableFontFamilyNames() };
    let family_names = unsafe { family_names.cast_unchecked::<CFString>() };
    for family_name in family_names.iter() {
        callback(family_name.to_string());
    }
}

fn font_template_for_local_font_descriptor(
    family_descriptor: CFRetained<CTFontDescriptor>,
) -> Option<FontTemplate> {
    let url = unsafe {
        family_descriptor
            .attribute(kCTFontURLAttribute)?
            .downcast::<CFURL>()
            .ok()?
    };
    let font_name = unsafe {
        family_descriptor
            .attribute(kCTFontNameAttribute)?
            .downcast::<CFString>()
            .ok()?
    };
    let traits = unsafe {
        family_descriptor
            .attribute(kCTFontTraitsAttribute)?
            .downcast::<CFDictionary>()
            .ok()?
    };
    let identifier = LocalFontIdentifier {
        postscript_name: Atom::from(font_name.to_string()),
        path: Atom::from(url.to_file_path()?.to_str()?),
    };
    Some(FontTemplate::new(
        FontIdentifier::Local(identifier),
        font_template_descriptor_from_ctfont_attributes(traits),
        None,
        None,
    ))
}

pub(crate) fn for_each_variation<F>(family_name: &str, mut callback: F)
where
    F: FnMut(FontTemplate),
{
    debug!("Looking for faces of family: {}", family_name);

    let specified_attributes: CFRetained<CFDictionary<CFString, CFType>> =
        CFDictionary::from_slices(
            &[unsafe { kCTFontFamilyNameAttribute }],
            &[CFString::from_str(family_name).as_ref()],
        );
    let wildcard_descriptor =
        unsafe { CTFontDescriptor::with_attributes(specified_attributes.as_ref()) };

    let values = [unsafe { kCTFontFamilyNameAttribute }];
    let values = values.as_ptr().cast::<*const c_void>().cast_mut();
    let mandatory_attributes = unsafe { CFSet::new(None, values, 1, std::ptr::null()) };
    let Some(mandatory_attributes) = mandatory_attributes else {
        return;
    };

    let matched_descriptors =
        unsafe { wildcard_descriptor.matching_font_descriptors(Some(&mandatory_attributes)) };
    let Some(matched_descriptors) = matched_descriptors else {
        return;
    };
    let matched_descriptors = unsafe { matched_descriptors.cast_unchecked::<CTFontDescriptor>() };

    for family_descriptor in matched_descriptors.iter() {
        if let Some(font_template) = font_template_for_local_font_descriptor(family_descriptor) {
            callback(font_template)
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
            // In Japanese typography, it is not common to use different fonts
            // for Kanji(Han), Hiragana, and Katakana within the same document. Since Hiragino supports
            // a comprehensive set of Japanese kanji, we uniformly fallback to Hiragino for all Japanese text.
            _ if options.lang == Some(String::from("ja")) => {
                families.push("Hiragino Sans");
                families.push("Hiragino Kaku Gothic ProN");
            },
            // CJK-related script codes are a bit troublesome because of unification;
            // we'll probably just get HAN much of the time, so the choice of which
            // language font to try for fallback is rather arbitrary. Usually, though,
            // we hope that font prefs will have handled this earlier.
            _ if matches!(script, Script::Bopomofo | Script::Han) &&
                options.lang != Some(String::from("ja")) =>
            {
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

    add_noto_fallback_families(options.clone(), &mut families);

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

pub(crate) fn default_system_generic_font_family(
    generic: GenericFontFamily,
) -> LowercaseFontFamilyName {
    match generic {
        GenericFontFamily::None | GenericFontFamily::Serif => "Times",
        GenericFontFamily::SansSerif => "Helvetica",
        GenericFontFamily::Monospace => "Menlo",
        GenericFontFamily::Cursive => "Apple Chancery",
        GenericFontFamily::Fantasy => "Papyrus",
        GenericFontFamily::SystemUi => "Helvetica",
    }
    .into()
}
