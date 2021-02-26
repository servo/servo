/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssconditionrule::CSSConditionRule;
use crate::dom::cssrule::SpecificCSSRule;
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::parser::ParserContext;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::supports_rule::SupportsCondition;
use style::stylesheets::{CssRuleType, Origin, SupportsRule};
use style_traits::{ParsingMode, ToCss};

#[dom_struct]
pub struct CSSSupportsRule {
    cssconditionrule: CSSConditionRule,
    #[ignore_malloc_size_of = "Arc"]
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

    /// <https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface>
    pub fn set_condition_text(&self, text: DOMString) {
        let mut input = ParserInput::new(&text);
        let mut input = Parser::new(&mut input);
        let cond = SupportsCondition::parse(&mut input);
        if let Ok(cond) = cond {
            let global = self.global();
            let win = global.as_window();
            let url = win.Document().url();
            let quirks_mode = win.Document().quirks_mode();
            let context = ParserContext::new(
                Origin::Author,
                &url,
                Some(CssRuleType::Supports),
                ParsingMode::DEFAULT,
                quirks_mode,
                None,
                None,
            );
            let enabled = {
                let namespaces = self
                    .cssconditionrule
                    .parent_stylesheet()
                    .style_stylesheet()
                    .contents
                    .namespaces
                    .read();
                cond.eval(&context, &namespaces)
            };
            let mut guard = self.cssconditionrule.shared_lock().write();
            let rule = self.supportsrule.write_with(&mut guard);
            rule.condition = cond;
            rule.enabled = enabled;
        }
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
