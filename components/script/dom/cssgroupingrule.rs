/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use style::shared_lock::SharedRwLock;
use style::stylesheets::{CssRuleType, CssRuleTypes};

use crate::dom::bindings::codegen::Bindings::CSSGroupingRuleBinding::CSSGroupingRuleMethods;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssconditionrule::CSSConditionRule;
use crate::dom::csslayerblockrule::CSSLayerBlockRule;
use crate::dom::cssrule::CSSRule;
use crate::dom::cssrulelist::{CSSRuleList, RulesSource};
use crate::dom::cssstylerule::CSSStyleRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSGroupingRule {
    cssrule: CSSRule,
    rulelist: MutNullableDom<CSSRuleList>,
}

impl CSSGroupingRule {
    pub(crate) fn new_inherited(parent_stylesheet: &CSSStyleSheet) -> CSSGroupingRule {
        CSSGroupingRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            rulelist: MutNullableDom::new(None),
        }
    }

    fn rulelist(&self, can_gc: CanGc) -> DomRoot<CSSRuleList> {
        let parent_stylesheet = self.upcast::<CSSRule>().parent_stylesheet();
        self.rulelist.or_init(|| {
            let rules = if let Some(rule) = self.downcast::<CSSConditionRule>() {
                rule.clone_rules()
            } else if let Some(rule) = self.downcast::<CSSLayerBlockRule>() {
                rule.clone_rules()
            } else if let Some(rule) = self.downcast::<CSSStyleRule>() {
                rule.ensure_rules()
            } else {
                unreachable!()
            };
            CSSRuleList::new(
                self.global().as_window(),
                parent_stylesheet,
                RulesSource::Rules(rules),
                can_gc,
            )
        })
    }

    pub(crate) fn parent_stylesheet(&self) -> &CSSStyleSheet {
        self.cssrule.parent_stylesheet()
    }

    pub(crate) fn shared_lock(&self) -> &SharedRwLock {
        self.cssrule.shared_lock()
    }
}

impl CSSGroupingRuleMethods<crate::DomTypeHolder> for CSSGroupingRule {
    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-cssrules
    fn CssRules(&self, can_gc: CanGc) -> DomRoot<CSSRuleList> {
        // XXXManishearth check origin clean flag
        self.rulelist(can_gc)
    }

    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-insertrule
    fn InsertRule(&self, rule: DOMString, index: u32, can_gc: CanGc) -> Fallible<u32> {
        // TODO: this should accumulate the rule types of all ancestors.
        let rule_type = self.cssrule.as_specific().ty();
        let containing_rule_types = CssRuleTypes::from(rule_type);
        let parse_relative_rule_type = match rule_type {
            CssRuleType::Style | CssRuleType::Scope => Some(rule_type),
            _ => None,
        };
        self.rulelist(can_gc).insert_rule(
            &rule,
            index,
            containing_rule_types,
            parse_relative_rule_type,
            can_gc,
        )
    }

    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-deleterule
    fn DeleteRule(&self, index: u32, can_gc: CanGc) -> ErrorResult {
        self.rulelist(can_gc).remove_rule(index)
    }
}
