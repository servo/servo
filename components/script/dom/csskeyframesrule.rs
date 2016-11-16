/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::KeyframesRule;

#[dom_struct]
pub struct CSSKeyframesRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    keyframesrule: Arc<RwLock<KeyframesRule>>,
}

impl CSSKeyframesRule {
    fn new_inherited(parent: &CSSStyleSheet, keyframesrule: Arc<RwLock<KeyframesRule>>) -> CSSKeyframesRule {
        CSSKeyframesRule {
            cssrule: CSSRule::new_inherited(parent),
            keyframesrule: keyframesrule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: &CSSStyleSheet,
               keyframesrule: Arc<RwLock<KeyframesRule>>) -> Root<CSSKeyframesRule> {
        reflect_dom_object(box CSSKeyframesRule::new_inherited(parent, keyframesrule),
                           window,
                           CSSKeyframesRuleBinding::Wrap)
    }
}

impl SpecificCSSRule for CSSKeyframesRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::KEYFRAMES_RULE
    }

    fn get_css(&self) -> DOMString {
        // self.keyframesrule.read().to_css_string().into()
        "".into()
    }
}
