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
use dom_struct::dom_struct;
use style::parser::{LengthParsingMode, ParserContext};
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylearc::Arc;
use style::stylesheets::{CssRuleType, SupportsRule};
use style::supports::SupportsCondition;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSSupportsRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_heap_size_of = "Arc"]
    supportsrule: Arc<Locked<SupportsRule>>,
}

impl CSSSupportsRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, supportsrule: Arc<Locked<SupportsRule>>)
                     -> CSSSupportsRule {
        let guard = parent_stylesheet.shared_lock().read();
        let list = supportsrule.read_with(&guard).rules.clone();
        CSSSupportsRule {
            cssconditionrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            supportsrule: supportsrule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               supportsrule: Arc<Locked<SupportsRule>>) -> Root<CSSSupportsRule> {
        reflect_dom_object(box CSSSupportsRule::new_inherited(parent_stylesheet, supportsrule),
                           window,
                           CSSSupportsRuleBinding::Wrap)
    }

    /// https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface
    pub fn get_condition_text(&self) -> DOMString {
        let guard = self.cssconditionrule.shared_lock().read();
        let rule = self.supportsrule.read_with(&guard);
        rule.condition.to_css_string().into()
    }

    /// https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface
    pub fn set_condition_text(&self, text: DOMString) {
        let mut input = Parser::new(&text);
        let cond = SupportsCondition::parse(&mut input);
        if let Ok(cond) = cond {
            let global = self.global();
            let win = global.as_window();
            let url = win.Document().url();
            let quirks_mode = win.Document().quirks_mode();
            let context = ParserContext::new_for_cssom(&url, win.css_error_reporter(), Some(CssRuleType::Supports),
                                                       LengthParsingMode::Default,
                                                       quirks_mode);
            let enabled = cond.eval(&context);
            let mut guard = self.cssconditionrule.shared_lock().write();
            let rule = self.supportsrule.write_with(&mut guard);
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
        let guard = self.cssconditionrule.shared_lock().read();
        self.supportsrule.read_with(&guard).to_css_string(&guard).into()
    }
}
