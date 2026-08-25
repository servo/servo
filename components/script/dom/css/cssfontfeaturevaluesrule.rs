/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::dom::MutNullableDom;
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_arc::Arc;
use style::shared_lock::ToCssWithGuard;
use style::stylesheets::{CssRuleType, FontFeatureValuesRule};
use style_traits::ToCss;

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSFontFeatureValuesRuleBinding::CSSFontFeatureValuesRuleMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::cssfontfeaturevaluesmap::CSSFontFeatureValuesMap;
use crate::dom::cssgroupingrule::CSSGroupingRule;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct CSSFontFeatureValuesRule {
    css_rule: CSSRule,

    /// A reference to the `@font-feature-values` rule as it is stored by stylo.
    #[no_trace]
    #[conditional_malloc_size_of]
    font_feature_values_rule: Arc<FontFeatureValuesRule>,

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-annotation>
    annotation: MutNullableDom<CSSFontFeatureValuesMap>,

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-ornaments>
    ornaments: MutNullableDom<CSSFontFeatureValuesMap>,

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-stylistic>
    stylistic: MutNullableDom<CSSFontFeatureValuesMap>,

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-swash>
    swash: MutNullableDom<CSSFontFeatureValuesMap>,

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-charactervariant>
    character_variant: MutNullableDom<CSSFontFeatureValuesMap>,

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-styleset>
    styleset: MutNullableDom<CSSFontFeatureValuesMap>,
}

impl CSSFontFeatureValuesRule {
    fn new_inherited(
        parent_rule: Option<&CSSGroupingRule>,
        parent_stylesheet: &CSSStyleSheet,
        font_feature_values_rule: Arc<FontFeatureValuesRule>,
    ) -> CSSFontFeatureValuesRule {
        CSSFontFeatureValuesRule {
            css_rule: CSSRule::new_inherited(parent_rule, parent_stylesheet),
            font_feature_values_rule,
            annotation: Default::default(),
            ornaments: Default::default(),
            stylistic: Default::default(),
            swash: Default::default(),
            character_variant: Default::default(),
            styleset: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        window: &Window,
        parent_rule: Option<&CSSGroupingRule>,
        parent_stylesheet: &CSSStyleSheet,
        font_feature_values_rule: Arc<FontFeatureValuesRule>,
    ) -> DomRoot<CSSFontFeatureValuesRule> {
        reflect_dom_object_with_cx(
            Box::new(CSSFontFeatureValuesRule::new_inherited(
                parent_rule,
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

impl CSSFontFeatureValuesRuleMethods<crate::DomTypeHolder> for CSSFontFeatureValuesRule {
    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-fontfamily>
    fn FontFamily(&self) -> DOMString {
        self.font_feature_values_rule
            .family_names
            .to_css_string()
            .into()
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-annotation>
    fn Annotation(&self, cx: &mut JSContext) -> DomRoot<CSSFontFeatureValuesMap> {
        self.annotation.or_init(|| {
            let global = self.global();
            CSSFontFeatureValuesMap::build_from(
                cx,
                &global,
                &self.font_feature_values_rule.annotation,
            )
        })
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-ornaments>
    fn Ornaments(&self, cx: &mut JSContext) -> DomRoot<CSSFontFeatureValuesMap> {
        self.ornaments.or_init(|| {
            let global = self.global();
            CSSFontFeatureValuesMap::build_from(
                cx,
                &global,
                &self.font_feature_values_rule.ornaments,
            )
        })
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-stylistic>
    fn Stylistic(&self, cx: &mut JSContext) -> DomRoot<CSSFontFeatureValuesMap> {
        self.stylistic.or_init(|| {
            let global = self.global();
            CSSFontFeatureValuesMap::build_from(
                cx,
                &global,
                &self.font_feature_values_rule.stylistic,
            )
        })
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-swash>
    fn Swash(&self, cx: &mut JSContext) -> DomRoot<CSSFontFeatureValuesMap> {
        self.swash.or_init(|| {
            let global = self.global();
            CSSFontFeatureValuesMap::build_from(cx, &global, &self.font_feature_values_rule.swash)
        })
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-charactervariant>
    fn CharacterVariant(&self, cx: &mut JSContext) -> DomRoot<CSSFontFeatureValuesMap> {
        self.character_variant.or_init(|| {
            let global = self.global();
            CSSFontFeatureValuesMap::build_from(
                cx,
                &global,
                &self.font_feature_values_rule.character_variant,
            )
        })
    }

    /// <https://drafts.csswg.org/css-fonts/#dom-cssfontfeaturevaluesrule-styleset>
    fn Styleset(&self, cx: &mut JSContext) -> DomRoot<CSSFontFeatureValuesMap> {
        self.styleset.or_init(|| {
            let global = self.global();
            CSSFontFeatureValuesMap::build_from(
                cx,
                &global,
                &self.font_feature_values_rule.styleset,
            )
        })
    }
}
