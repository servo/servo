/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::{CssRuleType, FontFaceRule};

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct CSSFontFaceRule {
    css_rule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    font_face_rule: RefCell<Arc<Locked<FontFaceRule>>>,
}

impl CSSFontFaceRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
    ) -> CSSFontFaceRule {
        CSSFontFaceRule {
            css_rule: CSSRule::new_inherited(parent_stylesheet),
            font_face_rule: RefCell::new(fontfacerule),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        fontfacerule: Arc<Locked<FontFaceRule>>,
    ) -> DomRoot<CSSFontFaceRule> {
        reflect_dom_object_with_cx(
            Box::new(CSSFontFaceRule::new_inherited(
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
