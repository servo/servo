/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSRuleBinding;
use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssfontfacerule::CSSFontFaceRule;
use dom::csskeyframesrule::CSSKeyframesRule;
use dom::cssmediarule::CSSMediaRule;
use dom::cssnamespacerule::CSSNamespaceRule;
use dom::cssstylerule::CSSStyleRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::cssviewportrule::CSSViewportRule;
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
        if let Some(rule) = self.downcast::<CSSStyleRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSFontFaceRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSKeyframesRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSMediaRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSNamespaceRule>() {
            rule as &SpecificCSSRule
        } else if let Some(rule) = self.downcast::<CSSViewportRule>() {
            rule as &SpecificCSSRule
        } else {
            unreachable!()
        }
    }

    // Given a StyleCSSRule, create a new instance of a derived class of
    // CSSRule based on which rule it is
    pub fn new_specific(window: &Window, parent: &CSSStyleSheet,
                        rule: StyleCSSRule) -> Root<CSSRule> {
        // be sure to update the match in as_specific when this is updated
        match rule {
            StyleCSSRule::Style(s) => Root::upcast(CSSStyleRule::new(window, parent, s)),
            StyleCSSRule::FontFace(s) => Root::upcast(CSSFontFaceRule::new(window, parent, s)),
            StyleCSSRule::Keyframes(s) => Root::upcast(CSSKeyframesRule::new(window, parent, s)),
            StyleCSSRule::Media(s) => Root::upcast(CSSMediaRule::new(window, parent, s)),
            StyleCSSRule::Namespace(s) => Root::upcast(CSSNamespaceRule::new(window, parent, s)),
            StyleCSSRule::Viewport(s) => Root::upcast(CSSViewportRule::new(window, parent, s)),
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
    fn SetCssText(&self, _: DOMString) {
        // do nothing
    }
}

pub trait SpecificCSSRule {
    fn ty(&self) -> u16;
    fn get_css(&self) -> DOMString;
}
