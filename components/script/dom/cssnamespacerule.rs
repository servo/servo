/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, NamespaceRule};

use crate::dom::bindings::codegen::Bindings::CSSNamespaceRuleBinding::CSSNamespaceRuleMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrule::{CSSRule, SpecificCSSRule};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSNamespaceRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    namespacerule: Arc<NamespaceRule>,
}

impl CSSNamespaceRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        namespacerule: Arc<NamespaceRule>,
    ) -> CSSNamespaceRule {
        CSSNamespaceRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            namespacerule,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        namespacerule: Arc<NamespaceRule>,
    ) -> DomRoot<CSSNamespaceRule> {
        reflect_dom_object(
            Box::new(CSSNamespaceRule::new_inherited(
                parent_stylesheet,
                namespacerule,
            )),
            window,
            CanGc::note(),
        )
    }
}

impl CSSNamespaceRuleMethods<crate::DomTypeHolder> for CSSNamespaceRule {
    // https://drafts.csswg.org/cssom/#dom-cssnamespacerule-prefix
    fn Prefix(&self) -> DOMString {
        self.namespacerule
            .prefix
            .as_ref()
            .map(|s| s.to_string().into())
            .unwrap_or_default()
    }

    // https://drafts.csswg.org/cssom/#dom-cssnamespacerule-namespaceuri
    fn NamespaceURI(&self) -> DOMString {
        (**self.namespacerule.url).into()
    }
}

impl SpecificCSSRule for CSSNamespaceRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Namespace
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.namespacerule.to_css_string(&guard).into()
    }
}
