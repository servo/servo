/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSConditionRuleBinding::CSSConditionRuleMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::str::DOMString;
use dom::cssgroupingrule::CSSGroupingRule;
use dom::cssmediarule::CSSMediaRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::csssupportsrule::CSSSupportsRule;
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{SharedRwLock, Locked};
use style::stylesheets::CssRules as StyleCssRules;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct CSSConditionRule<TH: TypeHolderTrait> {
    cssgroupingrule: CSSGroupingRule<TH>,
}

impl<TH: TypeHolderTrait> CSSConditionRule<TH> {
    pub fn new_inherited(parent_stylesheet: &CSSStyleSheet<TH>,
                         rules: Arc<Locked<StyleCssRules>>) -> CSSConditionRule<TH> {
        CSSConditionRule {
            cssgroupingrule: CSSGroupingRule::new_inherited(parent_stylesheet, rules),
        }
    }

    pub fn parent_stylesheet(&self) -> &CSSStyleSheet<TH> {
        self.cssgroupingrule.parent_stylesheet()
    }

    pub fn shared_lock(&self) -> &SharedRwLock {
        self.cssgroupingrule.shared_lock()
    }
}

impl<TH: TypeHolderTrait> CSSConditionRuleMethods for CSSConditionRule<TH> {
    /// <https://drafts.csswg.org/css-conditional-3/#dom-cssconditionrule-conditiontext>
    fn ConditionText(&self) -> DOMString {
        if let Some(rule) = self.downcast::<CSSMediaRule<TH>>() {
            rule.get_condition_text()
        } else if let Some(rule) = self.downcast::<CSSSupportsRule<TH>>() {
            rule.get_condition_text()
        } else {
            unreachable!()
        }
    }

    /// <https://drafts.csswg.org/css-conditional-3/#dom-cssconditionrule-conditiontext>
    fn SetConditionText(&self, text: DOMString) {
        if let Some(rule) = self.downcast::<CSSMediaRule<TH>>() {
            rule.set_condition_text(text)
        } else if let Some(rule) = self.downcast::<CSSSupportsRule<TH>>() {
            rule.set_condition_text(text)
        } else {
            unreachable!()
        }
    }
}
