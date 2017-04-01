/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Bindings for CSS Rule objects

use font_face::{FontFaceData, Source};
use gecko_bindings::bindings;
use gecko_bindings::structs::{self, CSSFontFaceDescriptors, nsCSSFontFaceRule};
use gecko_bindings::sugar::refptr::{RefPtr, UniqueRefPtr};
use shared_lock::{ToCssWithGuard, SharedRwLockReadGuard};
use std::fmt;

/// A @font-face rule
pub type FontFaceRule = RefPtr<nsCSSFontFaceRule>;

fn set_font_face_descriptors(descriptors: &mut CSSFontFaceDescriptors,
                             data: FontFaceData) {
    // font-family
    descriptors.mFamily.set_string_from_atom(&data.family.name);

    macro_rules! map_enum {
        ($target:ident = ($data:ident: $prop:ident) {
            $($servo:ident => $gecko:ident,)+
        }) => {{
            use computed_values::$prop::T;
            descriptors.$target.set_enum(match data.$data {
                $( T::$servo => structs::$gecko as i32, )+
            })
        }}
    }

    // font-style
    map_enum!(mStyle = (style: font_style) {
        normal => NS_FONT_STYLE_NORMAL,
        italic => NS_FONT_STYLE_ITALIC,
        oblique => NS_FONT_STYLE_OBLIQUE,
    });

    // font-weight
    descriptors.mWeight.set_integer(data.weight as i32);

    // font-stretch
    map_enum!(mStretch = (stretch: font_stretch) {
        normal          => NS_FONT_STRETCH_NORMAL,
        ultra_condensed => NS_FONT_STRETCH_ULTRA_CONDENSED,
        extra_condensed => NS_FONT_STRETCH_EXTRA_CONDENSED,
        condensed       => NS_FONT_STRETCH_CONDENSED,
        semi_condensed  => NS_FONT_STRETCH_SEMI_CONDENSED,
        semi_expanded   => NS_FONT_STRETCH_SEMI_EXPANDED,
        expanded        => NS_FONT_STRETCH_EXPANDED,
        extra_expanded  => NS_FONT_STRETCH_EXTRA_EXPANDED,
        ultra_expanded  => NS_FONT_STRETCH_ULTRA_EXPANDED,
    });

    // src
    let src_len = data.sources.iter().fold(0, |acc, src| {
        acc + match *src {
            // Each format hint takes one position in the array of mSrc.
            Source::Url(ref url) => url.format_hints.len() + 1,
            Source::Local(_) => 1,
        }
    });
    let mut target_srcs =
        descriptors.mSrc.set_array(src_len as i32).as_mut_slice().iter_mut();
    macro_rules! next { () => {
        target_srcs.next().expect("Length of target_srcs should be enough")
    } }
    for src in data.sources.iter() {
        match *src {
            Source::Url(ref url) => {
                next!().set_url(&url.url);
                for hint in url.format_hints.iter() {
                    next!().set_font_format(&hint);
                }
            }
            Source::Local(ref family) => {
                next!().set_local_font(&family.name);
            }
        }
    }
    debug_assert!(target_srcs.next().is_none(), "Should have filled all slots");

    // unicode-range
    let target_ranges = descriptors.mUnicodeRange
        .set_array((data.unicode_range.len() * 2) as i32)
        .as_mut_slice().chunks_mut(2);
    for (range, target) in data.unicode_range.iter().zip(target_ranges) {
        target[0].set_integer(range.start as i32);
        target[1].set_integer(range.end as i32);
    }

    // The following three descriptors are not implemented yet.
    // font-feature-settings
    descriptors.mFontFeatureSettings.set_normal();
    // font-language-override
    descriptors.mFontLanguageOverride.set_normal();
    // font-display
    descriptors.mDisplay.set_enum(structs::NS_FONT_DISPLAY_AUTO as i32);
}

impl From<FontFaceData> for FontFaceRule {
    fn from(data: FontFaceData) -> FontFaceRule {
        let mut result = unsafe {
            UniqueRefPtr::from_addrefed(bindings::Gecko_CSSFontFaceRule_Create())
        };
        set_font_face_descriptors(&mut result.mDecl.mDescriptors, data);
        result.get()
    }
}

impl ToCssWithGuard for FontFaceRule {
    fn to_css<W>(&self, _guard: &SharedRwLockReadGuard, dest: &mut W) -> fmt::Result
    where W: fmt::Write {
        ns_auto_string!(css_text);
        unsafe {
            bindings::Gecko_CSSFontFaceRule_GetCssText(self.get(), &mut *css_text);
        }
        write!(dest, "{}", css_text)
    }
}
