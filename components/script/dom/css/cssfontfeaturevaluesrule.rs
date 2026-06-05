/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, FontFeatureValuesRule};

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct CSSFontFeatureValuesRule {
    css_rule: CSSRule,
    #[no_trace]
    #[conditional_malloc_size_of]
    font_feature_values_rule: Arc<FontFeatureValuesRule>,
}

impl CSSFontFeatureValuesRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        font_feature_values_rule: Arc<FontFeatureValuesRule>,
    ) -> CSSFontFeatureValuesRule {
        CSSFontFeatureValuesRule {
            css_rule: CSSRule::new_inherited(parent_stylesheet),
            font_feature_values_rule,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        font_feature_values_rule: Arc<FontFeatureValuesRule>,
    ) -> DomRoot<CSSFontFeatureValuesRule> {
        reflect_dom_object_with_cx(
            Box::new(CSSFontFeatureValuesRule::new_inherited(
                parent_stylesheet,
                font_feature_values_rule,
            )),
            window,
            cx,
        )
    }
}

impl SpecificCSSRule for CSSFontFeatureValuesRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::FontFeatureValues
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_rule.shared_lock().read();
        self.font_feature_values_rule.to_css_string(&guard).into()
    }
}
