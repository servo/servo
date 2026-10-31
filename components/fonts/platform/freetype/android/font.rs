/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::sync::Arc;

use fonts_traits::LocalFontIdentifier;
use ndk::font::FontMatcher;
use style::values::computed::font::SingleFontFamily::{FamilyName, Generic};
use style::values::computed::font::{FontStyle, FontWeight, FontWidth};
use stylo_atoms::Atom;

use crate::platform::font_list::default_system_generic_font_family;
use crate::{
    FallbackFontSelectionOptions, Font, FontIdentifier, FontRef, FontTemplate,
    FontTemplateDescriptor, FontTemplateRef, LowercaseFontFamilyName,
};

impl Font {
    pub(crate) fn find_fallback_using_system_font_api(
        &self,
        options: &FallbackFontSelectionOptions,
    ) -> Option<FontRef> {
        let mut a_font_matcher = FontMatcher::new();
        let mut codepoint = [0u16; 2];
        let codepoint = options.character.encode_utf16(&mut codepoint[..]);
        let Some(preferred_font_families) = options.preferred_font_families else {
            return None;
        };

        for font_family in preferred_font_families {
            let generic_family_name: LowercaseFontFamilyName;
            let family_name_string = match &font_family.family_descriptor.family {
                FamilyName(family) => family.name.as_str(),
                Generic(generic_font) => {
                    generic_family_name = default_system_generic_font_family(*generic_font);
                    &generic_family_name
                },
            };

            let Ok(family_name_cstring) = CString::new(&*family_name_string) else {
                continue;
            };
            let family_name = &family_name_cstring;

            let ndk_font = a_font_matcher.match_font(family_name, &codepoint, None);

            let Some(path) = ndk_font.path().to_str() else {
                continue;
            };
            let identifier = LocalFontIdentifier {
                path: Atom::from(path),
                // According to https://docs.rs/ndk/latest/ndk/font/struct.Font.html,
                // "This always returns 0 if the target font file is a regular font."
                // So despite the function's name, this function will handle `.ttf` or `.otf` files just fine.
                face_index: ndk_font.collection_index() as u16,
                named_instance_index: 0, // TODO: Find a way to properly set this value. NDK 0.9.0 doesn't expose this information.
            };

            let font_weight = FontWeight::from_float(ndk_font.weight().to_u16() as f32);
            let font_style = if ndk_font.is_italic() {
                FontStyle::ITALIC
            } else {
                FontStyle::NORMAL
            };

            // TODO(rtjkro): In the future, find a way to properly set this value,
            // possibly using the width axis if it exists in the font.
            let font_width = FontWidth::NORMAL;

            let font_template_descriptor =
                FontTemplateDescriptor::new(font_weight, font_width, font_style);

            let template = FontTemplate::new(
                FontIdentifier::Local(identifier),
                font_template_descriptor,
                None,
            );
            let template = FontTemplateRef::new(template);

            return Some(FontRef(Arc::new(
                Font::new(template, self.descriptor.clone(), None, None).ok()?,
            )));
        }

        None
    }
}
