/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Bindings for CSS Rule objects

use computed_values::{font_style, font_weight, font_stretch};
use computed_values::font_family::FamilyName;
use cssparser::UnicodeRange;
use font_face::{FontFaceRuleData, Source};
use gecko_bindings::bindings;
use gecko_bindings::structs::{self, nsCSSFontFaceRule, nsCSSValue};
use gecko_bindings::sugar::ns_css_value::ToNsCssValue;
use gecko_bindings::sugar::refptr::{RefPtr, UniqueRefPtr};
use shared_lock::{ToCssWithGuard, SharedRwLockReadGuard};
use std::fmt;

/// A @font-face rule
pub type FontFaceRule = RefPtr<nsCSSFontFaceRule>;

impl ToNsCssValue for FamilyName {
    fn convert(&self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_string_from_atom(&self.name)
    }
}

impl ToNsCssValue for font_weight::T {
    fn convert(&self, nscssvalue: &mut nsCSSValue) {
        nscssvalue.set_integer(*self as i32)
    }
}

macro_rules! map_enum {
    (
        $(
            $prop:ident {
                $($servo:ident => $gecko:ident,)+
            }
        )+
    ) => {
        $(
            impl ToNsCssValue for $prop::T {
                fn convert(&self, nscssvalue: &mut nsCSSValue) {
                    nscssvalue.set_enum(match *self {
                        $( $prop::T::$servo => structs::$gecko as i32, )+
                    })
                }
            }
        )+
    }
}

map_enum! {
    font_style {
        normal => NS_FONT_STYLE_NORMAL,
        italic => NS_FONT_STYLE_ITALIC,
        oblique => NS_FONT_STYLE_OBLIQUE,
    }

    font_stretch {
        normal          => NS_FONT_STRETCH_NORMAL,
        ultra_condensed => NS_FONT_STRETCH_ULTRA_CONDENSED,
        extra_condensed => NS_FONT_STRETCH_EXTRA_CONDENSED,
        condensed       => NS_FONT_STRETCH_CONDENSED,
        semi_condensed  => NS_FONT_STRETCH_SEMI_CONDENSED,
        semi_expanded   => NS_FONT_STRETCH_SEMI_EXPANDED,
        expanded        => NS_FONT_STRETCH_EXPANDED,
        extra_expanded  => NS_FONT_STRETCH_EXTRA_EXPANDED,
        ultra_expanded  => NS_FONT_STRETCH_ULTRA_EXPANDED,
    }
}

impl ToNsCssValue for Vec<Source> {
    fn convert(&self, nscssvalue: &mut nsCSSValue) {
        let src_len = self.iter().fold(0, |acc, src| {
            acc + match *src {
                // Each format hint takes one position in the array of mSrc.
                Source::Url(ref url) => url.format_hints.len() + 1,
                Source::Local(_) => 1,
            }
        });
        let mut target_srcs =
            nscssvalue.set_array(src_len as i32).as_mut_slice().iter_mut();
        macro_rules! next { () => {
            target_srcs.next().expect("Length of target_srcs should be enough")
        } }
        for src in self.iter() {
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
    }
}

impl ToNsCssValue for Vec<UnicodeRange> {
    fn convert(&self, nscssvalue: &mut nsCSSValue) {
        let target_ranges = nscssvalue
            .set_array((self.len() * 2) as i32)
            .as_mut_slice().chunks_mut(2);
        for (range, target) in self.iter().zip(target_ranges) {
            target[0].set_integer(range.start as i32);
            target[1].set_integer(range.end as i32);
        }
    }
}

impl From<FontFaceRuleData> for FontFaceRule {
    fn from(data: FontFaceRuleData) -> FontFaceRule {
        let mut result = unsafe {
            UniqueRefPtr::from_addrefed(bindings::Gecko_CSSFontFaceRule_Create())
        };
        data.set_descriptors(&mut result.mDecl.mDescriptors);
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
