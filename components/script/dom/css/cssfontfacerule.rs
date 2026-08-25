/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::dom::MutNullableDom;
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_arc::Arc;
use style::font_face::DescriptorId;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::{CssRuleType, FontFaceRule};

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSFontFaceRuleBinding::CSSFontFaceRuleMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::css::cssfontfacedescriptors::CSSFontFaceDescriptors;
use crate::dom::cssgroupingrule::CSSGroupingRule;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct CSSFontFaceRule {
    css_rule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    font_face_rule: RefCell<Arc<Locked<FontFaceRule>>>,
    descriptors: MutNullableDom<CSSFontFaceDescriptors>,
}

impl CSSFontFaceRule {
    fn new_inherited(
        parent_rule: Option<&CSSGroupingRule>,
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
    ) -> CSSFontFaceRule {
        CSSFontFaceRule {
            css_rule: CSSRule::new_inherited(parent_rule, parent_stylesheet),
            font_face_rule: RefCell::new(fontfacerule),
            descriptors: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        parent_rule: Option<&CSSGroupingRule>,
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
    ) -> DomRoot<CSSFontFaceRule> {
        reflect_dom_object_with_cx(
            Box::new(CSSFontFaceRule::new_inherited(
                parent_rule,
                parent_stylesheet,
                fontfacerule,
            )),
            window,
            cx,
        )
    }

    pub(crate) fn update_rule(&self, fontfacerule: Arc<Locked<FontFaceRule>>) {
        *self.font_face_rule.borrow_mut() = fontfacerule;
    }

    /// Retrieve the value of a given descriptor in the `@font-face` rule.
    pub(crate) fn get_descriptor(&self, descriptor_id: DescriptorId) -> DOMString {
        let guard = self.css_rule.shared_lock().read();
        let mut result = String::new();
        self.font_face_rule
            .borrow()
            .read_with(&guard)
            .descriptors
            .get(descriptor_id, &mut result)
            .expect("Writing to a string should never fail");

        result.into()
    }

    /// Return the value n'th existing descriptor in the `@font-face` rule.
    pub(crate) fn get_descriptor_by_index(&self, index: u32) -> Option<DOMString> {
        let guard = self.css_rule.shared_lock().read();
        let descriptor_id_at_index = self
            .font_face_rule
            .borrow()
            .read_with(&guard)
            .descriptors
            .at(index as usize)?;
        Some(self.get_descriptor(descriptor_id_at_index))
    }

    /// Return the number of descriptors in this `@font-face` rule.
    pub(crate) fn descriptor_length(&self) -> usize {
        let guard = self.css_rule.shared_lock().read();
        self.font_face_rule
            .borrow()
            .read_with(&guard)
            .descriptors
            .len()
    }
}

impl SpecificCSSRule for CSSFontFaceRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::FontFace
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_rule.shared_lock().read();
        self.font_face_rule
            .borrow()
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}

impl CSSFontFaceRuleMethods<crate::DomTypeHolder> for CSSFontFaceRule {
    fn Style(&self, cx: &mut JSContext) -> DomRoot<CSSFontFaceDescriptors> {
        self.descriptors.or_init(|| {
            let global = self.css_rule.global();
            CSSFontFaceDescriptors::new(cx, &global, self)
        })
    }
}
