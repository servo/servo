/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSConditionRuleBinding::CSSConditionRuleMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::cssrule::CSSRule;
use dom::cssrulelist::{CSSRuleList, RulesSource};
use dom::cssstylesheet::CSSStyleSheet;
use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::CssRules as StyleCssRules;

#[dom_struct]
pub struct CSSConditionRule {
    cssgroupingrule: CSSGroupingRule,
}

impl CSSConditionRule {
    pub fn new_inherited(parent_stylesheet: &CSSStyleSheet,
                         rules: Arc<RwLock<StyleCssRules>>) -> CSSConditionRule {
        CSSConditionRule {
            cssgroupingrule: CSSGroupingRule::new_inherited(parent_stylesheet, rules),
        }
    }

}

impl CSSConditionRuleMethods for CSSConditionRule {
    fn ConditionText(&self) -> DOMString {
        "".into()
    }
}
