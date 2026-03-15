/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::sync::Arc;

use fonts_traits::LocalFontIdentifier;
use ndk::font::FontMatcher;
use style::values::computed::font::SingleFontFamily::{FamilyName, Generic};
use style::values::computed::font::{FontStretch, FontStyle, FontWeight, GenericFontFamily};
use stylo_atoms::Atom;

use crate::{
    Font, FontGroup, FontIdentifier, FontRef, FontTemplate, FontTemplateDescriptor, FontTemplateRef,
};

impl FontGroup {
    pub fn find_using_system_font_api(&self, character: char) -> Option<FontRef> {
        let mut a_font_matcher = FontMatcher::new();
        let codepoint_vec: Vec<u16> = character.to_string().encode_utf16().collect();
        let codepoint: &[u16] = &codepoint_vec;

        for font_family in &self.families {
            let family_name_string = match &font_family.family_descriptor.family {
                FamilyName(family) => family.name.to_string(),
                Generic(generic_font) => process_generic_font(generic_font),
            };

            if family_name_string == "" {
                continue; // This means that the generic font is not of a type parseable by Servo. In this case, skip.
            }
            let family_name_str = family_name_string.as_str();
            let family_name_cstring =
                CString::new(family_name_str).expect("String contains interior null bytes");
            let family_name = family_name_cstring.as_c_str();
            let mut text_run_length = 0;

            let ndk_font =
                a_font_matcher.match_font(family_name, codepoint, Some(&mut text_run_length));

            if text_run_length != 0 {
                let identifier = LocalFontIdentifier {
                    path: Atom::from(
                        ndk_font
                            .path()
                            .to_str()
                            .expect("Failed to convert path to string!")
                            .to_string(),
                    ),
                    face_index: ndk_font.axis_count() as u16,
                    named_instance_index: 0, // Ignore for non-variable-fonts.
                };

                let font_weight = FontWeight::from_float(ndk_font.weight().to_u16() as f32);
                let font_style = if ndk_font.is_italic() {
                    FontStyle::ITALIC
                } else {
                    FontStyle::NORMAL
                };

                // TODO(richardtjokroutomo): In the future, find a way to properly set this value,
                // possibly using the width axis if it exists in the font.
                let mut font_stretch = FontStretch::NORMAL;

                let font_template_descriptor =
                    FontTemplateDescriptor::new(font_weight, font_stretch, font_style);

                let template = FontTemplate::new(
                    FontIdentifier::Local(identifier),
                    font_template_descriptor,
                    None,
                    None,
                );
                let template = FontTemplateRef::new(template);

                return Some(FontRef(Arc::new(
                    Font::new(template, self.descriptor.clone(), None, None).ok()?,
                )));
            }
        }

        None
    }
}

fn process_generic_font(generic_font: &GenericFontFamily) -> String {
    match generic_font {
        GenericFontFamily::Serif => "Serif".to_string(),
        GenericFontFamily::SansSerif => "SansSerif".to_string(),
        GenericFontFamily::Monospace => "Monospace".to_string(),
        GenericFontFamily::Cursive => "Cursive".to_string(),
        GenericFontFamily::Fantasy => "Fantasy".to_string(),
        _ => "".to_string(),
    }
}
