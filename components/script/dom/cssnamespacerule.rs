/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSNamespaceRuleBinding;
use dom::bindings::codegen::Bindings::CSSNamespaceRuleBinding::CSSNamespaceRuleMethods;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::NamespaceRule;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct CSSNamespaceRule<TH: TypeHolderTrait> {
    cssrule: CSSRule<TH>,
    #[ignore_malloc_size_of = "Arc"]
    namespacerule: Arc<Locked<NamespaceRule>>,
}

impl<TH: TypeHolderTrait> CSSNamespaceRule<TH> {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet<TH>, namespacerule: Arc<Locked<NamespaceRule>>)
                     -> Self {
        CSSNamespaceRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            namespacerule: namespacerule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, parent_stylesheet: &CSSStyleSheet<TH>,
               namespacerule: Arc<Locked<NamespaceRule>>) -> DomRoot<Self> {
        reflect_dom_object(Box::new(CSSNamespaceRule::new_inherited(parent_stylesheet, namespacerule)),
                           window,
                           CSSNamespaceRuleBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> CSSNamespaceRuleMethods for CSSNamespaceRule<TH> {
    // https://drafts.csswg.org/cssom/#dom-cssnamespacerule-prefix
    fn Prefix(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.namespacerule.read_with(&guard).prefix
            .as_ref().map(|s| s.to_string().into())
            .unwrap_or(DOMString::new())
    }

    // https://drafts.csswg.org/cssom/#dom-cssnamespacerule-namespaceuri
    fn NamespaceURI(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        (*self.namespacerule.read_with(&guard).url).into()
    }
}

impl<TH: TypeHolderTrait> SpecificCSSRule for CSSNamespaceRule<TH> {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::NAMESPACE_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.namespacerule.read_with(&guard).to_css_string(&guard).into()
    }
}
