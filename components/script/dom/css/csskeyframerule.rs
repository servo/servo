/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, SharedRwLockReadGuard, ToCssWithGuard};
use style::stylesheets::CssRuleType;
use style::stylesheets::keyframes_rule::Keyframe;

use super::cssrule::{CSSRule, SpecificCSSRule};
use super::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use super::cssstylesheet::CSSStyleSheet;
use crate::dom::bindings::codegen::Bindings::CSSKeyframeRuleBinding::CSSKeyframeRuleMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSKeyframeRule {
    css_rule: CSSRule,
    #[ignore_malloc_size_of = "Stylo"]
    #[no_trace]
    keyframe_rule: RefCell<Arc<Locked<Keyframe>>>,
    style_declaration: MutNullableDom<CSSStyleDeclaration>,
}

impl CSSKeyframeRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        keyframerule: Arc<Locked<Keyframe>>,
    ) -> CSSKeyframeRule {
        CSSKeyframeRule {
            css_rule: CSSRule::new_inherited(parent_stylesheet),
            keyframe_rule: RefCell::new(keyframerule),
            style_declaration: Default::default(),
        }
    }

    pub(crate) fn new(
        window: &Window,
        parent_stylesheet: &CSSStyleSheet,
        keyframerule: Arc<Locked<Keyframe>>,
        can_gc: CanGc,
    ) -> DomRoot<CSSKeyframeRule> {
        reflect_dom_object(
            Box::new(CSSKeyframeRule::new_inherited(
                parent_stylesheet,
                keyframerule,
            )),
            window,
            can_gc,
        )
    }

    pub(crate) fn update_rule(
        &self,
        keyframerule: Arc<Locked<Keyframe>>,
        guard: &SharedRwLockReadGuard,
    ) {
        if let Some(ref style_decl) = self.style_declaration.get() {
            style_decl.update_property_declaration_block(&keyframerule.read_with(guard).block);
        }
        *self.keyframe_rule.borrow_mut() = keyframerule;
    }
}

impl CSSKeyframeRuleMethods<crate::DomTypeHolder> for CSSKeyframeRule {
    /// <https://drafts.csswg.org/css-animations/#dom-csskeyframerule-style>
    fn Style(&self, can_gc: CanGc) -> DomRoot<CSSStyleDeclaration> {
        self.style_declaration.or_init(|| {
            let guard = self.css_rule.shared_lock().read();
            CSSStyleDeclaration::new(
                self.global().as_window(),
                CSSStyleOwner::CSSRule(
                    Dom::from_ref(self.upcast()),
                    RefCell::new(self.keyframe_rule.borrow().read_with(&guard).block.clone()),
                ),
                None,
                CSSModificationAccess::ReadWrite,
                can_gc,
            )
        })
    }
}

impl SpecificCSSRule for CSSKeyframeRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Keyframe
    }

    fn get_css(&self) -> DOMString {
        let guard = self.css_rule.shared_lock().read();
        self.keyframe_rule
            .borrow()
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}
