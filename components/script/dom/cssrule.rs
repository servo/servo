/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSRuleBinding;
use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssstylesheet::CSSStyleSheet;
use dom::cssstylerule::CSSStyleRule;
use dom::window::Window;
use style::stylesheets::CSSRule as StyleCSSRule;


#[dom_struct]
pub struct CSSRule {
    reflector_: Reflector,
    parent: MutNullableHeap<JS<CSSStyleSheet>>,
}

impl CSSRule {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(parent: &CSSStyleSheet) -> CSSRule {
        CSSRule {
            reflector_: Reflector::new(),
            parent: MutNullableHeap::new(Some(parent)),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: &CSSStyleSheet) -> Root<CSSRule> {
        reflect_dom_object(box CSSRule::new_inherited(parent),
                           window,
                           CSSRuleBinding::Wrap)
    }

    pub fn as_specific(&self) -> &SpecificCSSRule {
        if let Some(stylerule) = self.downcast::<CSSStyleRule>() {
            stylerule as &SpecificCSSRule
        } else {
            unreachable!()
        }
    }

    // Given a StyleCSSRule, create a new instance of a derived class of
    // CSSRule based on which rule it is
    pub fn new_specific(window: &Window, parent: &CSSStyleSheet,
                        rule: StyleCSSRule) -> Root<CSSRule> {
        match rule {
            StyleCSSRule::Style(s) => Root::upcast(CSSStyleRule::new(window, parent, s)),
            _ => unimplemented!()
        }
    }
}

impl CSSRuleMethods for CSSRule {
    // https://drafts.csswg.org/cssom/#dom-cssrule-type
    fn Type(&self) -> u16 {
        self.as_specific().ty()
    }

    // https://drafts.csswg.org/cssom/#dom-cssrule-parentstylesheet
    fn GetParentStyleSheet(&self) -> Option<Root<CSSStyleSheet>> {
        self.parent.get()
    }

    // https://drafts.csswg.org/cssom/#dom-cssrule-csstext
    fn CssText(&self) -> DOMString {
        self.as_specific().get_css()
    }

    // https://drafts.csswg.org/cssom/#dom-cssrule-csstext
    fn SetCssText(&self, s: DOMString) {
        // do nothing
    }
}

pub trait SpecificCSSRule {
    fn ty(&self) -> u16;
    fn get_css(&self) -> DOMString;
}
