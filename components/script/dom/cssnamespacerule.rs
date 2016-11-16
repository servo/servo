/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSNamespaceRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::NamespaceRule;

#[dom_struct]
pub struct CSSNamespaceRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    namespacerule: Arc<RwLock<NamespaceRule>>,
}

impl CSSNamespaceRule {
    fn new_inherited(parent: &CSSStyleSheet, namespacerule: Arc<RwLock<NamespaceRule>>) -> CSSNamespaceRule {
        CSSNamespaceRule {
            cssrule: CSSRule::new_inherited(parent),
            namespacerule: namespacerule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: &CSSStyleSheet,
               namespacerule: Arc<RwLock<NamespaceRule>>) -> Root<CSSNamespaceRule> {
        reflect_dom_object(box CSSNamespaceRule::new_inherited(parent, namespacerule),
                           window,
                           CSSNamespaceRuleBinding::Wrap)
    }
}

impl SpecificCSSRule for CSSNamespaceRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::NAMESPACE_RULE
    }

    fn get_css(&self) -> DOMString {
        // self.namespacerule.read().to_css_string().into()
        "".into()
    }
}
