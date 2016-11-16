/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::StyleRule;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSStyleRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    stylerule: Arc<RwLock<StyleRule>>,
}

impl CSSStyleRule {
    fn new_inherited(parent: &CSSStyleSheet, stylerule: Arc<RwLock<StyleRule>>) -> CSSStyleRule {
        CSSStyleRule {
            cssrule: CSSRule::new_inherited(parent),
            stylerule: stylerule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: &CSSStyleSheet,
               stylerule: Arc<RwLock<StyleRule>>) -> Root<CSSStyleRule> {
        reflect_dom_object(box CSSStyleRule::new_inherited(parent, stylerule),
                           window,
                           CSSStyleRuleBinding::Wrap)
    }
}

impl SpecificCSSRule for CSSStyleRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::STYLE_RULE
    }

    fn get_css(&self) -> DOMString {
        self.stylerule.read().to_css_string().into()
    }
}
