/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSGroupingRuleBinding::CSSGroupingRuleMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::cssrule::CSSRule;
use dom::cssrulelist::{CSSRuleList, RulesSource};
use dom::cssstylesheet::CSSStyleSheet;
use dom_struct::dom_struct;
use style::shared_lock::{SharedRwLock, Locked};
use style::stylearc::Arc;
use style::stylesheets::CssRules as StyleCssRules;

#[dom_struct]
pub struct CSSGroupingRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    rules: Arc<Locked<StyleCssRules>>,
    rulelist: MutNullableJS<CSSRuleList>,
}

impl CSSGroupingRule {
    pub fn new_inherited(parent_stylesheet: &CSSStyleSheet,
                         rules: Arc<Locked<StyleCssRules>>) -> CSSGroupingRule {
        CSSGroupingRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            rules: rules,
            rulelist: MutNullableJS::new(None),
        }
    }

    fn rulelist(&self) -> Root<CSSRuleList> {
        let parent_stylesheet = self.upcast::<CSSRule>().parent_stylesheet();
        self.rulelist.or_init(|| CSSRuleList::new(self.global().as_window(),
                                                  parent_stylesheet,
                                                  RulesSource::Rules(self.rules.clone())))
    }

    pub fn parent_stylesheet(&self) -> &CSSStyleSheet {
        self.cssrule.parent_stylesheet()
    }

    pub fn shared_lock(&self) -> &SharedRwLock {
        self.cssrule.shared_lock()
    }
}

impl CSSGroupingRuleMethods for CSSGroupingRule {
    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-cssrules
    fn CssRules(&self) -> Root<CSSRuleList> {
        // XXXManishearth check origin clean flag
        self.rulelist()
    }

    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-insertrule
    fn InsertRule(&self, rule: DOMString, index: u32) -> Fallible<u32> {
        self.rulelist().insert_rule(&rule, index, /* nested */ true)
    }

    // https://drafts.csswg.org/cssom/#dom-cssgroupingrule-deleterule
    fn DeleteRule(&self, index: u32) -> ErrorResult {
        self.rulelist().remove_rule(index)
    }
}
