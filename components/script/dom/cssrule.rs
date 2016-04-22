/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSRuleBinding;
use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::window::Window;
use style::selector_impl::ServoSelectorImpl;
use style::stylesheets;


#[dom_struct]
pub struct CSSRule {
    reflector_: Reflector,
    rule: stylesheets::CSSRule<ServoSelectorImpl>,
}

impl CSSRule {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(rule: stylesheets::CSSRule<ServoSelectorImpl>) -> CSSRule {
        CSSRule {
            reflector_: Reflector::new(),
            rule: rule
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, rule: stylesheets::CSSRule<ServoSelectorImpl>) -> Root<CSSRule> {
        reflect_dom_object(box CSSRule::new_inherited(rule),
                           GlobalRef::Window(window),
                           CSSRuleBinding::Wrap)
    }
}

impl CSSRuleMethods for CSSRule {
    // https://drafts.csswg.org/cssom/#dom-cssrule-type
    fn Type_(&self) -> u16 {
        match &self.rule {
           CSSStyleRule => CSSRuleConstants::STYLE_RULE,
        }
        //TODO match for other rules
    }
}
