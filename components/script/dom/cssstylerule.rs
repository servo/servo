/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleRuleBinding;
use dom::bindings::codegen::Bindings::CSSStyleRuleBinding::CSSStyleRuleMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::cssrule::CSSRule;
use dom::window::Window;
use std::string::String;
use style::stylesheets;
use util::str::DOMString;

#[dom_struct]
pub struct CSSStyleRule {
    cssrule: CSSRule,
    selectortext: DOMString,
}

impl CSSStyleRule {
    #[allow(unrooted_must_root)]
    fn new_inherited(selectortext: DOMString) -> CSSStyleRule {
        CSSStyleRule {
            cssrule: CSSRule::new_inherited(stylesheets::CSSRule::Charset(String::from(selectortext.clone()))),
            selectortext: selectortext,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, selectortext: DOMString) -> Root<CSSStyleRule> {
        reflect_dom_object(box CSSStyleRule::new_inherited(selectortext),
                           GlobalRef::Window(window),
                           CSSStyleRuleBinding::Wrap)
    }
}

impl CSSStyleRuleMethods for CSSStyleRule {
    // https://drafts.csswg.org/cssom/#dom-cssstylerule-selectortext
    fn SelectorText(&self) -> DOMString {
         self.selectortext.clone()
    }
}
