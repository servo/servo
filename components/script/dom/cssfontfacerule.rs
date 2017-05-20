/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSFontFaceRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylearc::Arc;
use style::stylesheets::FontFaceRule;

#[dom_struct]
pub struct CSSFontFaceRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    fontfacerule: Arc<Locked<FontFaceRule>>,
}

impl CSSFontFaceRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, fontfacerule: Arc<Locked<FontFaceRule>>)
                     -> CSSFontFaceRule {
        CSSFontFaceRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            fontfacerule: fontfacerule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               fontfacerule: Arc<Locked<FontFaceRule>>) -> Root<CSSFontFaceRule> {
        reflect_dom_object(box CSSFontFaceRule::new_inherited(parent_stylesheet, fontfacerule),
                           window,
                           CSSFontFaceRuleBinding::Wrap)
    }
}

impl SpecificCSSRule for CSSFontFaceRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::FONT_FACE_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.fontfacerule.read_with(&guard).to_css_string(&guard).into()
    }
}
