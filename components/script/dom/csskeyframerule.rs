/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSKeyframeRuleBinding::{self, CSSKeyframeRuleMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::keyframes_rule::Keyframe;

#[dom_struct]
pub struct CSSKeyframeRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    keyframerule: Arc<Locked<Keyframe>>,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
}

impl CSSKeyframeRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, keyframerule: Arc<Locked<Keyframe>>)
                     -> CSSKeyframeRule {
        CSSKeyframeRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            keyframerule: keyframerule,
            style_decl: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               keyframerule: Arc<Locked<Keyframe>>) -> DomRoot<CSSKeyframeRule> {
        reflect_dom_object(Box::new(CSSKeyframeRule::new_inherited(parent_stylesheet, keyframerule)),
                           window,
                           CSSKeyframeRuleBinding::Wrap)
    }
}

impl CSSKeyframeRuleMethods for CSSKeyframeRule {
    // https://drafts.csswg.org/css-animations/#dom-csskeyframerule-style
    fn Style(&self) -> DomRoot<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let guard = self.cssrule.shared_lock().read();
            CSSStyleDeclaration::new(
                self.global().as_window(),
                CSSStyleOwner::CSSRule(
                    Dom::from_ref(self.upcast()),
                    self.keyframerule.read_with(&guard).block.clone(),
                ),
                None,
                CSSModificationAccess::ReadWrite,
            )
        })
    }
}

impl SpecificCSSRule for CSSKeyframeRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::KEYFRAME_RULE
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.keyframerule.read_with(&guard).to_css_string(&guard).into()
    }
}
