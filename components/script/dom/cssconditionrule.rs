/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::CssRules as StyleCssRules;

use crate::dom::bindings::codegen::Bindings::CSSConditionRuleBinding::CSSConditionRuleMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssgroupingrule::CSSGroupingRule;
use crate::dom::cssmediarule::CSSMediaRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::csssupportsrule::CSSSupportsRule;

#[dom_struct]
pub(crate) struct CSSConditionRule {
    cssgroupingrule: CSSGroupingRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    rules: Arc<Locked<StyleCssRules>>,
}

impl CSSConditionRule {
    pub(crate) fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        rules: Arc<Locked<StyleCssRules>>,
    ) -> CSSConditionRule {
        CSSConditionRule {
            cssgroupingrule: CSSGroupingRule::new_inherited(parent_stylesheet),
            rules,
        }
    }

    pub(crate) fn parent_stylesheet(&self) -> &CSSStyleSheet {
        self.cssgroupingrule.parent_stylesheet()
    }

    pub(crate) fn shared_lock(&self) -> &SharedRwLock {
        self.cssgroupingrule.shared_lock()
    }

    pub(crate) fn clone_rules(&self) -> Arc<Locked<StyleCssRules>> {
        self.rules.clone()
    }
}

impl CSSConditionRuleMethods<crate::DomTypeHolder> for CSSConditionRule {
    /// <https://drafts.csswg.org/css-conditional-3/#dom-cssconditionrule-conditiontext>
    fn ConditionText(&self) -> DOMString {
        if let Some(rule) = self.downcast::<CSSMediaRule>() {
            rule.get_condition_text()
        } else if let Some(rule) = self.downcast::<CSSSupportsRule>() {
            rule.get_condition_text()
        } else {
            unreachable!()
        }
    }
}
