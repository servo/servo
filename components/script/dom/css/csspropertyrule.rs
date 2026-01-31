/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, PropertyRule};
use style_traits::ToCss;

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSPropertyRuleBinding::CSSPropertyRuleMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSPropertyRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    property_rule: RefCell<Arc<PropertyRule>>,
}

impl CSSPropertyRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, property_rule: Arc<PropertyRule>) -> Self {
        CSSPropertyRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            property_rule: RefCell::new(property_rule),
        }
    }

    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        property_rule: Arc<PropertyRule>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(parent_stylesheet, property_rule)),
            window,
            can_gc,
        )
    }

    pub(crate) fn update_rule(&self, property_rule: Arc<PropertyRule>) {
        *self.property_rule.borrow_mut() = property_rule;
    }
}

impl SpecificCSSRule for CSSPropertyRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Property
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.property_rule.borrow().to_css_string(&guard).into()
    }
}

impl CSSPropertyRuleMethods<crate::DomTypeHolder> for CSSPropertyRule {
    /// <https://drafts.css-houdini.org/css-properties-values-api/#dom-csspropertyrule-name>
    fn Name(&self) -> DOMString {
        format!("--{}", self.property_rule.borrow().name.0).into()
    }

    /// <https://drafts.css-houdini.org/css-properties-values-api/#dom-csspropertyrule-syntax>
    fn Syntax(&self) -> DOMString {
        self.property_rule
            .borrow()
            .data
            .syntax
            .specified_string()
            .unwrap_or_else(|| {
                debug_assert!(false, "PropertyRule exists but missing a syntax string?");
                "*"
            })
            .into()
    }

    /// <https://drafts.css-houdini.org/css-properties-values-api/#dom-csspropertyrule-initialvalue>
    fn GetInitialValue(&self) -> Option<DOMString> {
        self.property_rule
            .borrow()
            .data
            .initial_value
            .as_ref()
            .map(|value| value.to_css_string().into())
    }

    /// <https://drafts.css-houdini.org/css-properties-values-api/#dom-csspropertyrule-inherits>
    fn Inherits(&self) -> bool {
        self.property_rule.borrow().inherits()
    }
}
