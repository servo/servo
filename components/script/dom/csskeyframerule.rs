/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_arc::Arc;
use style::shared_lock::{Locked, ToCssWithGuard};
use style::stylesheets::keyframes_rule::Keyframe;
use style::stylesheets::CssRuleType;

use crate::dom::bindings::codegen::Bindings::CSSKeyframeRuleBinding::CSSKeyframeRuleMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::cssrule::{CSSRule, SpecificCSSRule};
use crate::dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct CSSKeyframeRule {
    cssrule: CSSRule,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    keyframerule: Arc<Locked<Keyframe>>,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
}

impl CSSKeyframeRule {
    fn new_inherited(
        parent_stylesheet: &CSSStyleSheet,
        keyframerule: Arc<Locked<Keyframe>>,
    ) -> CSSKeyframeRule {
        CSSKeyframeRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            keyframerule,
            style_decl: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
}

impl CSSKeyframeRuleMethods<crate::DomTypeHolder> for CSSKeyframeRule {
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
                CanGc::note(),
            )
        })
    }
}

impl SpecificCSSRule for CSSKeyframeRule {
    fn ty(&self) -> CssRuleType {
        CssRuleType::Keyframe
    }

    fn get_css(&self) -> DOMString {
        let guard = self.cssrule.shared_lock().read();
        self.keyframerule
            .read_with(&guard)
            .to_css_string(&guard)
            .into()
    }
}
