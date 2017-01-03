/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use dom::bindings::codegen::Bindings::CSSSupportsRuleBinding;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssconditionrule::CSSConditionRule;
use dom::cssrule::SpecificCSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::parser::ParserContext;
use style::stylesheets::SupportsRule;
use style::supports::SupportsCondition;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSSupportsRule {
    cssrule: CSSConditionRule,
    #[ignore_heap_size_of = "Arc"]
    supportsrule: Arc<RwLock<SupportsRule>>,
}

impl CSSSupportsRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, supportsrule: Arc<RwLock<SupportsRule>>)
                     -> CSSSupportsRule {
        let list = supportsrule.read().rules.clone();
        CSSSupportsRule {
            cssrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            supportsrule: supportsrule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               supportsrule: Arc<RwLock<SupportsRule>>) -> Root<CSSSupportsRule> {
        reflect_dom_object(box CSSSupportsRule::new_inherited(parent_stylesheet, supportsrule),
                           window,
                           CSSSupportsRuleBinding::Wrap)
    }

    /// https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface
    pub fn get_condition_text(&self) -> DOMString {
        let rule = self.supportsrule.read();
        rule.condition.to_css_string().into()
    }

    /// https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface
    pub fn set_condition_text(&self, text: DOMString) {
        let mut input = Parser::new(&text);
        let cond = SupportsCondition::parse(&mut input);
        if let Ok(cond) = cond {
            let url = self.global().as_window().Document().url();
            let context = ParserContext::new_for_cssom(&url);
            let enabled = cond.eval(&context);
            let mut rule = self.supportsrule.write();
            rule.condition = cond;
            rule.enabled = enabled;
        }
    }
}

impl SpecificCSSRule for CSSSupportsRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::SUPPORTS_RULE
    }

    fn get_css(&self) -> DOMString {
        self.supportsrule.read().to_css_string().into()
    }
}
