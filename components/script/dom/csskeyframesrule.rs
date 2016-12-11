/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::Parser;
use dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding;
use dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding::CSSKeyframesRuleMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::csskeyframerule::CSSKeyframeRule;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssrulelist::{CSSRuleList, RulesSource};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::keyframes::{Keyframe, KeyframeSelector};
use style::parser::ParserContextExtraData;
use style::stylesheets::KeyframesRule;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSKeyframesRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    keyframesrule: Arc<RwLock<KeyframesRule>>,
    rulelist: MutNullableJS<CSSRuleList>,
}

impl CSSKeyframesRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, keyframesrule: Arc<RwLock<KeyframesRule>>)
                     -> CSSKeyframesRule {
        CSSKeyframesRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            keyframesrule: keyframesrule,
            rulelist: MutNullableJS::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               keyframesrule: Arc<RwLock<KeyframesRule>>) -> Root<CSSKeyframesRule> {
        reflect_dom_object(box CSSKeyframesRule::new_inherited(parent_stylesheet, keyframesrule),
                           window,
                           CSSKeyframesRuleBinding::Wrap)
    }

    fn rulelist(&self) -> Root<CSSRuleList> {
        self.rulelist.or_init(|| {
            let parent_stylesheet = &self.upcast::<CSSRule>().parent_stylesheet();
            CSSRuleList::new(self.global().as_window(),
                             parent_stylesheet,
                             RulesSource::Keyframes(self.keyframesrule.clone()))
        })
    }

    /// Given a keyframe selector, finds the index of the first corresponding rule if any
    fn find_rule(&self, selector: &str) -> Option<usize> {
        let mut input = Parser::new(selector);
        if let Ok(sel) = KeyframeSelector::parse(&mut input) {
            // This finds the *last* element matching a selector
            // because that's the rule that applies. Thus, rposition
            self.keyframesrule.read()
                .keyframes.iter().rposition(|frame| {
                    frame.read().selector == sel
                })
        } else {
            None
        }
    }
}

impl CSSKeyframesRuleMethods for CSSKeyframesRule {
    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-cssrules
    fn CssRules(&self) -> Root<CSSRuleList> {
        self.rulelist()
    }

    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-appendrule
    fn AppendRule(&self, rule: DOMString) {
        let rule = Keyframe::parse(&rule, self.cssrule.parent_stylesheet().style_stylesheet(),
                                   ParserContextExtraData::default());
        if let Ok(rule) = rule {
            self.keyframesrule.write().keyframes.push(rule);
            self.rulelist().append_lazy_dom_rule();
        }
    }

    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-deleterule
    fn DeleteRule(&self, selector: DOMString) {
        if let Some(idx) = self.find_rule(&selector) {
            let _ = self.rulelist().remove_rule(idx as u32);
        }
    }

    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-findrule
    fn FindRule(&self, selector: DOMString) -> Option<Root<CSSKeyframeRule>> {
        self.find_rule(&selector).and_then(|idx| {
            self.rulelist().item(idx as u32)
        }).and_then(Root::downcast)
    }
}

impl SpecificCSSRule for CSSKeyframesRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::KEYFRAMES_RULE
    }

    fn get_css(&self) -> DOMString {
        self.keyframesrule.read().to_css_string().into()
    }

    fn deparent_children(&self) {
        self.rulelist.get().map(|list| list.deparent_all());
    }
}
