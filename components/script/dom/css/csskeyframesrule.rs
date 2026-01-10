/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, SharedRwLockReadGuard, ToCssWithGuard};
use style::stylesheets::keyframes_rule::{Keyframe, KeyframeSelector, KeyframesRule};
use style::stylesheets::{CssRuleType, StylesheetInDocument};
use style::values::KeyframesName;

use super::csskeyframerule::CSSKeyframeRule;
use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssrulelist::{CSSRuleList, RulesSource};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding::CSSKeyframesRuleMethods;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSKeyframesRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    keyframesrule: RefCell<Arc<Locked<KeyframesRule>>>,
    rulelist: MutNullableDom<CSSRuleList>,
}

impl CSSKeyframesRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        keyframesrule: Arc<Locked<KeyframesRule>>,
    ) -> CSSKeyframesRule {
        CSSKeyframesRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            keyframesrule: RefCell::new(keyframesrule),
            rulelist: MutNullableDom::new(None),
        }
    }

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

    fn rulelist(&self, can_gc: CanGc) -> DomRoot<CSSRuleList> {
        self.rulelist.or_init(|| {
            let parent_stylesheet = &self.upcast::<CSSRule>().parent_stylesheet();
            CSSRuleList::new(
                self.global().as_window(),
                parent_stylesheet,
                RulesSource::Keyframes(self.keyframesrule.borrow().clone()),
                can_gc,
            )
        })
    }

    /// Given a keyframe selector, finds the index of the first corresponding rule if any
    fn find_rule(&self, selector: &DOMString) -> Option<usize> {
        let selector = selector.str();
        let mut input = ParserInput::new(&selector);
        let mut input = Parser::new(&mut input);
        if let Ok(sel) = KeyframeSelector::parse(&mut input) {
            let guard = self.cssrule.shared_lock().read();
            // This finds the *last* element matching a selector
            // because that's the rule that applies. Thus, rposition
            self.keyframesrule
                .borrow()
                .read_with(&guard)
                .keyframes
                .iter()
                .rposition(|frame| frame.read_with(&guard).selector == sel)
        } else {
            None
        }
    }

    pub(crate) fn update_rule(
        &self,
        keyframesrule: Arc<Locked<KeyframesRule>>,
        guard: &SharedRwLockReadGuard,
    ) {
        if let Some(rulelist) = self.rulelist.get() {
            rulelist.update_rules(RulesSource::Keyframes(keyframesrule.clone()), guard);
        }

        *self.keyframesrule.borrow_mut() = keyframesrule;
    }
}

impl CSSKeyframesRuleMethods<crate::DomTypeHolder> for CSSKeyframesRule {
    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-cssrules>
    fn CssRules(&self, can_gc: CanGc) -> DomRoot<CSSRuleList> {
        self.rulelist(can_gc)
    }

    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-appendrule>
    fn AppendRule(&self, rule: DOMString, can_gc: CanGc) {
        let style_stylesheet = self.cssrule.parent_stylesheet().style_stylesheet();
        let rule = rule.str();
        let rule = {
            let guard = style_stylesheet.shared_lock.read();
            Keyframe::parse(
                &rule,
                style_stylesheet.contents(&guard),
                &style_stylesheet.shared_lock,
            )
        };

        if let Ok(rule) = rule {
            self.cssrule.parent_stylesheet().will_modify();
            let mut guard = self.cssrule.shared_lock().write();
            self.keyframesrule
                .borrow()
                .write_with(&mut guard)
                .keyframes
                .push(rule);
            self.rulelist(can_gc).append_lazy_dom_rule();
            self.cssrule.parent_stylesheet().notify_invalidations();
        }
    }

    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-deleterule>
    fn DeleteRule(&self, selector: DOMString, can_gc: CanGc) {
        if let Some(idx) = self.find_rule(&selector) {
            let _ = self.rulelist(can_gc).remove_rule(idx as u32);
        }
    }

    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-findrule>
    fn FindRule(&self, selector: DOMString, can_gc: CanGc) -> Option<DomRoot<CSSKeyframeRule>> {
        self.find_rule(&selector)
            .and_then(|idx| self.rulelist(can_gc).item(idx as u32, can_gc))
            .and_then(DomRoot::downcast)
    }

    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-name>
    fn Name(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        DOMString::from(&**self.keyframesrule.borrow().read_with(&guard).name.as_atom())
    }

    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframesrule-name>
    fn SetName(&self, value: DOMString) -> ErrorResult {
        // Spec deviation: https://github.com/w3c/csswg-drafts/issues/801
        // Setting this property to a CSS-wide keyword or `none` does not throw,
        // it stores a value that serializes as a quoted string.
        self.cssrule.parent_stylesheet().will_modify();
        let name = KeyframesName::from_ident(&value.str());
        let mut guard = self.cssrule.shared_lock().write();
        self.keyframesrule.borrow().write_with(&mut guard).name = name;
        self.cssrule.parent_stylesheet().notify_invalidations();
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
            .borrow()
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
