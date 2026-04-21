/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::context::JSContext;
use servo_arc::Arc;
use style::shared_lock::{SharedRwLockReadGuard, ToCssWithGuard};
use style::stylesheets::{CssRuleType, SupportsRule};
use style_traits::ToCss;

use super::cssconditionrule::CSSConditionRule;
use super::cssrule::SpecificCSSRule;
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::reflector::reflect_dom_object_with_cx;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct CSSSupportsRule {
    css_condition_rule: CSSConditionRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    supports_rule: RefCell<Arc<SupportsRule>>,
}

impl CSSSupportsRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        supportsrule: Arc<SupportsRule>,
    ) -> CSSSupportsRule {
        let list = supportsrule.rules.clone();
        CSSSupportsRule {
            css_condition_rule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            supports_rule: RefCell::new(supportsrule),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        supportsrule: Arc<SupportsRule>,
    ) -> DomRoot<CSSSupportsRule> {
        reflect_dom_object_with_cx(
            Box::new(CSSSupportsRule::new_inherited(
                parent_stylesheet,
                supportsrule,
            )),
            window,
            cx,
        )
    }

    /// <https://drafts.csswg.org/css-conditional-3/#the-csssupportsrule-interface>
    pub(crate) fn get_condition_text(&self) -> DOMString {
        self.supports_rule.borrow().condition.to_css_string().into()
    }

    pub(crate) fn update_rule(
        &self,
        supportsrule: Arc<SupportsRule>,
        guard: &SharedRwLockReadGuard,
    ) {
        self.css_condition_rule
            .update_rules(supportsrule.rules.clone(), guard);
        *self.supports_rule.borrow_mut() = supportsrule;
    }
}

impl SpecificCSSRule for CSSSupportsRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Supports
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_condition_rule.shared_lock().read();
        self.supports_rule.borrow().to_css_string(&guard).into()
    }
}
