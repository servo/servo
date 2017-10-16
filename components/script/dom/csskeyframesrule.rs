/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding;
use dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding::CSSKeyframesRuleMethods;
use dom::bindings::error::ErrorResult;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::csskeyframerule::CSSKeyframeRule;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssrulelist::{CSSRuleList, RulesSource};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::keyframes_rule::{KeyframesRule, Keyframe, KeyframeSelector};
use style::values::KeyframesName;

#[dom_struct]
pub struct CSSKeyframesRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    keyframesrule: Arc<Locked<KeyframesRule>>,
    rulelist: MutNullableDom<CSSRuleList>,
}

impl CSSKeyframesRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, keyframesrule: Arc<Locked<KeyframesRule>>)
                     -> CSSKeyframesRule {
        CSSKeyframesRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            keyframesrule: keyframesrule,
            rulelist: MutNullableDom::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               keyframesrule: Arc<Locked<KeyframesRule>>) -> DomRoot<CSSKeyframesRule> {
        reflect_dom_object(Box::new(CSSKeyframesRule::new_inherited(parent_stylesheet, keyframesrule)),
                           window,
                           CSSKeyframesRuleBinding::Wrap)
    }

    fn rulelist(&self) -> DomRoot<CSSRuleList> {
        self.rulelist.or_init(|| {
            let parent_stylesheet = &self.upcast::<CSSRule>().parent_stylesheet();
            CSSRuleList::new(self.global().as_window(),
                             parent_stylesheet,
                             RulesSource::Keyframes(self.keyframesrule.clone()))
        })
    }

    /// Given a keyframe selector, finds the index of the first corresponding rule if any
    fn find_rule(&self, selector: &str) -> Option<usize> {
        let mut input = ParserInput::new(selector);
        let mut input = Parser::new(&mut input);
        if let Ok(sel) = KeyframeSelector::parse(&mut input) {
            let guard = self.cssrule.shared_lock().read();
            // This finds the *last* element matching a selector
            // because that's the rule that applies. Thus, rposition
            self.keyframesrule.read_with(&guard)
                .keyframes.iter().rposition(|frame| {
                    frame.read_with(&guard).selector == sel
                })
        } else {
            None
        }
    }
}

impl CSSKeyframesRuleMethods for CSSKeyframesRule {
    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-cssrules
    fn CssRules(&self) -> DomRoot<CSSRuleList> {
        self.rulelist()
    }

    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-appendrule
    fn AppendRule(&self, rule: DOMString) {
        let style_stylesheet = self.cssrule.parent_stylesheet().style_stylesheet();
        let rule = Keyframe::parse(
            &rule,
            &style_stylesheet.contents,
            &style_stylesheet.shared_lock
        );

        if let Ok(rule) = rule {
            let mut guard = self.cssrule.shared_lock().write();
            self.keyframesrule.write_with(&mut guard).keyframes.push(rule);
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
    fn FindRule(&self, selector: DOMString) -> Option<DomRoot<CSSKeyframeRule>> {
        self.find_rule(&selector).and_then(|idx| {
            self.rulelist().item(idx as u32)
        }).and_then(DomRoot::downcast)
    }

    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-name
    fn Name(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        DOMString::from(&**self.keyframesrule.read_with(&guard).name.as_atom())
    }

    // https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-name
    fn SetName(&self, value: DOMString) -> ErrorResult {
        // Spec deviation: https://github.com/w3c/csswg-drafts/issues/801
        // Setting this property to a CSS-wide keyword or `none` does not throw,
        // it stores a value that serializes as a quoted string.
        let name = KeyframesName::from_ident(&value);
        let mut guard = self.cssrule.shared_lock().write();
        self.keyframesrule.write_with(&mut guard).name = name;
        Ok(())
    }
}

impl SpecificCSSRule for CSSKeyframesRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::KEYFRAMES_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.keyframesrule.read_with(&guard).to_css_string(&guard).into()
    }

    fn deparent_children(&self) {
        self.rulelist.get().map(|list| list.deparent_all());
    }
}
