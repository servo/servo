/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::SupportsRule;
use style_traits::ToCss;

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssconditionrule::CSSConditionRule;
use crate::dom::cssrule::SpecificCSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;

#[dom_struct]
pub struct CSSSupportsRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    supportsrule: Arc<Locked<SupportsRule>>,
}

impl CSSSupportsRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        supportsrule: Arc<Locked<SupportsRule>>,
    ) -> CSSSupportsRule {
        let guard = parent_stylesheet.shared_lock().read();
        let list = supportsrule.read_with(&guard).rules.clone();
        CSSSupportsRule {
            cssconditionrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            supportsrule: supportsrule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        supportsrule: Arc<Locked<SupportsRule>>,
    ) -> DomRoot<CSSSupportsRule> {
        reflect_dom_object(
            Box::new(CSSSupportsRule::new_inherited(
                parent_stylesheet,
                supportsrule,
            )),
            window,
        )
    }

    /// <https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface>
    pub fn get_condition_text(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        let rule = self.supportsrule.read_with(&guard);
        rule.condition.to_css_string().into()
    }
}

impl SpecificCSSRule for CSSSupportsRule {
    fn ty(&self) -> u16 {
        use crate::dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::SUPPORTS_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        self.supportsrule
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}
