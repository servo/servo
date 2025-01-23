/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, SupportsRule};
use style_traits::ToCss;

use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssconditionrule::CSSConditionRule;
use crate::dom::cssrule::SpecificCSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSSupportsRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    supportsrule: Arc<SupportsRule>,
}

impl CSSSupportsRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        supportsrule: Arc<SupportsRule>,
    ) -> CSSSupportsRule {
        let list = supportsrule.rules.clone();
        CSSSupportsRule {
            cssconditionrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            supportsrule,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        supportsrule: Arc<SupportsRule>,
    ) -> DomRoot<CSSSupportsRule> {
        reflect_dom_object(
            Box::new(CSSSupportsRule::new_inherited(
                parent_stylesheet,
                supportsrule,
            )),
            window,
            CanGc::note(),
        )
    }

    /// <https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface>
    pub(crate) fn get_condition_text(&self) -> DOMString {
        self.supportsrule.condition.to_css_string().into()
    }
}

impl SpecificCSSRule for CSSSupportsRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Supports
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        self.supportsrule.to_css_string(&guard).into()
    }
}
