/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::keyframes_rule::{Keyframe, KeyframeSelector, KeyframesRule};
use style::stylesheets::CssRuleType;
use style::values::KeyframesName;

use crate::dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding::CSSKeyframesRuleMethods;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::csskeyframerule::CSSKeyframeRule;
use crate::dom::cssrule::{CSSRule, SpecificCSSRule};
use crate::dom::cssrulelist::{CSSRuleList, RulesSource};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSKeyframesRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    keyframesrule: Arc<Locked<KeyframesRule>>,
    rulelist: MutNullableDom<CSSRuleList>,
}

impl CSSKeyframesRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        keyframesrule: Arc<Locked<KeyframesRule>>,
    ) -> CSSKeyframesRule {
        CSSKeyframesRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            keyframesrule,
            rulelist: MutNullableDom::new(None),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        keyframesrule: Arc<Locked<KeyframesRule>>,
        can_gc: CanGc,
    ) -> DomRoot<CSSKeyframesRule> {
        reflect_dom_object(
            Box::new(CSSKeyframesRule::new_inherited(
                parent_stylesheet,
                keyframesrule,
            )),
            window,
            can_gc,
        )
    }

    fn rulelist(&self) -> DomRoot<CSSRuleList> {
        self.rulelist.or_init(|| {
            let parent_stylesheet = &self.upcast::<CSSRule>().parent_stylesheet();
            CSSRuleList::new(
                self.global().as_window(),
                parent_stylesheet,
                RulesSource::Keyframes(self.keyframesrule.clone()),
                CanGc::note(),
            )
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
            self.keyframesrule
                .read_with(&guard)
                .keyframes
                .iter()
                .rposition(|frame| frame.read_with(&guard).selector == sel)
        } else {
            None
        }
    }
}

impl CSSKeyframesRuleMethods<crate::DomTypeHolder> for CSSKeyframesRule {
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
            &style_stylesheet.shared_lock,
        );

        if let Ok(rule) = rule {
            let mut guard = self.cssrule.shared_lock().write();
            self.keyframesrule
                .write_with(&mut guard)
                .keyframes
                .push(rule);
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
        self.find_rule(&selector)
            .and_then(|idx| self.rulelist().item(idx as u32))
            .and_then(DomRoot::downcast)
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
    fn ty(&self) -> CssRuleType {
        CssRuleType::Keyframes
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.keyframesrule
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }

    fn deparent_children(&self) {
        if let Some(list) = self.rulelist.get() {
            list.deparent_all()
        }
    }
}
