/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object;
use style::font_face::DescriptorId;

use crate::dom::GlobalScope;
use crate::dom::bindings::codegen::Bindings::CSSFontFaceDescriptorsBinding::CSSFontFaceDescriptorsMethods;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::RootedDOMString;
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
        reflect_dom_object(
            cx,
            Box::new(CSSFontFaceDescriptors::new_inherited(font_face_rule)),
            global,
        )
    }

    /// <https://drafts.csswg.org/cssom/#dom-cssstyledeclaration-getpropertyvalue>
    pub(crate) fn get_property_value(&self, property: &str) -> RootedDOMString {
        let Ok(descriptor_id) = DescriptorId::from_ident(property) else {
            return Default::default();
        };
        self.font_face_rule.get_descriptor(descriptor_id)
    }
}

impl CSSFontFaceDescriptorsMethods<crate::DomTypeHolder> for CSSFontFaceDescriptors {
    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-src>
    fn Src(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::Src)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontfamily>
    fn FontFamily(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontFamily)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-family>
    fn Font_family(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontFamily)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontstyle>
    fn FontStyle(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontStyle)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-style>
    fn Font_style(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontStyle)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontweight>
    fn FontWeight(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWeight)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-weight>
    fn Font_weight(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWeight)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontstretch>
    fn FontStretch(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWidth)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-stretch>
    fn Font_stretch(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWidth)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontwidth>
    fn FontWidth(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWidth)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-width>
    fn Font_width(&self) -> RootedDOMString {
        self.font_face_rule.get_descriptor(DescriptorId::FontWidth)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-unicoderange>
    fn UnicodeRange(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::UnicodeRange)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-unicode-range>
    fn Unicode_range(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::UnicodeRange)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontfeaturesettings>
    fn FontFeatureSettings(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontFeatureSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-feature-settings>
    fn Font_feature_settings(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontFeatureSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontvariationsettings>
    fn FontVariationSettings(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontVariationSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-variation-settings>
    fn Font_variation_settings(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontVariationSettings)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontdisplay>
    fn FontDisplay(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontDisplay)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-display>
    fn Font_display(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontDisplay)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-fontlanguageoverride>
    fn FontLanguageOverride(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontLanguageOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-font-language-override>
    fn Font_language_override(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::FontLanguageOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-ascentoverride>
    fn AscentOverride(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::AscentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-ascent-override>
    fn Ascent_override(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::AscentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-descentoverride>
    fn DescentOverride(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::DescentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-descent-override>
    fn Descent_override(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::DescentOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-linegapoverride>
    fn LineGapOverride(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::LineGapOverride)
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfacedescriptors-line-gap-override>
    fn Line_gap_override(&self) -> RootedDOMString {
        self.font_face_rule
            .get_descriptor(DescriptorId::LineGapOverride)
    }

    fn IndexedGetter(&self, index: u32) -> Option<RootedDOMString> {
        self.font_face_rule.get_descriptor_by_index(index)
    }
    fn Length(&self, _cx: &JSContext) -> u32 {
        self.font_face_rule.descriptor_length() as u32
    }
}
