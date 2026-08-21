/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;
use style::font_face::DescriptorId;

use crate::dom::GlobalScope;
use crate::dom::bindings::codegen::Bindings::CSSFontFaceDescriptorsBinding::CSSFontFaceDescriptorsMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::css::cssfontfacerule::CSSFontFaceRule;
use crate::dom::css::cssstyledeclaration::CSSStyleDeclaration;
use crate::dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleOwner};

#[dom_struct]
pub(crate) struct CSSFontFaceDescriptors {
    style_declaration: CSSStyleDeclaration,
    font_face_rule: Dom<CSSFontFaceRule>,
}

impl CSSFontFaceDescriptors {
    pub(crate) fn new_inherited(font_face_rule: &CSSFontFaceRule) -> CSSFontFaceDescriptors {
        CSSFontFaceDescriptors {
            // FIXME: Don't use CSSModificationAccess::Readonly (requires using something other than CSSStyleOwner::Null as well)
            style_declaration: CSSStyleDeclaration::new_inherited(
                CSSStyleOwner::Null,
                None,
                CSSModificationAccess::Readonly,
            ),
            font_face_rule: Dom::from_ref(font_face_rule),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        font_face_rule: &CSSFontFaceRule,
    ) -> DomRoot<CSSFontFaceDescriptors> {
        reflect_dom_object_with_cx(
            Box::new(CSSFontFaceDescriptors::new_inherited(font_face_rule)),
            global,
            cx,
        )
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertyvalue>
    pub(crate) fn get_property_value(&self, property: &str) -> DOMString {
        let Ok(descriptor_id) = DescriptorId::from_ident(property) else {
            return Default::default();
        };
        self.font_face_rule.get_descriptor(descriptor_id)
    }
}

impl CSSFontFaceDescriptorsMethods<crate::DomTypeHolder> for CSSFontFaceDescriptors {
    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-src>
    fn Src(&self) -> DOMString {
        self.font_face_rule.get_descriptor(DescriptorId::Src)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontfamily>
    fn FontFamily(&self) -> DOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontFamily)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-family>
    fn Font_family(&self) -> DOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontFamily)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontstyle>
    fn FontStyle(&self) -> DOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontStyle)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-style>
    fn Font_style(&self) -> DOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontStyle)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontweight>
    fn FontWeight(&self) -> DOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWeight)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-weight>
    fn Font_weight(&self) -> DOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWeight)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontstretch>
    fn FontStretch(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontStretch)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-stretch>
    fn Font_stretch(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontStretch)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontwidth>
    fn FontWidth(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontStretch)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-width>
    fn Font_width(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontStretch)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-unicoderange>
    fn UnicodeRange(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::UnicodeRange)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-unicode-range>
    fn Unicode_range(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::UnicodeRange)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontfeaturesettings>
    fn FontFeatureSettings(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontFeatureSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-feature-settings>
    fn Font_feature_settings(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontFeatureSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontvariationsettings>
    fn FontVariationSettings(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontVariationSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-variation-settings>
    fn Font_variation_settings(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontVariationSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontdisplay>
    fn FontDisplay(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontDisplay)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-display>
    fn Font_display(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontDisplay)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontlanguageoverride>
    fn FontLanguageOverride(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontLanguageOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-language-override>
    fn Font_language_override(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontLanguageOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-ascentoverride>
    fn AscentOverride(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::AscentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-ascent-override>
    fn Ascent_override(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::AscentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-descentoverride>
    fn DescentOverride(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::DescentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-descent-override>
    fn Descent_override(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::DescentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-linegapoverride>
    fn LineGapOverride(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::LineGapOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-line-gap-override>
    fn Line_gap_override(&self) -> DOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::LineGapOverride)
    }

    fn IndexedGetter(&self, index: u32) -> Option<DOMString> {
        self.font_face_rule.get_descriptor_by_index(index)
    }
    fn Length(&self, _cx: &JSContext) -> u32 {
        self.font_face_rule.descriptor_length() as u32
    }
}
