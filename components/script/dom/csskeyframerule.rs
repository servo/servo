/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSKeyframeRuleBinding::{self, CSSKeyframeRuleMethods};
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstyledeclaration::{CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::keyframes::Keyframe;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSKeyframeRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    keyframerule: Arc<RwLock<Keyframe>>,
    style_decl: MutNullableJS<CSSStyleDeclaration>,
}

impl CSSKeyframeRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, keyframerule: Arc<RwLock<Keyframe>>)
                     -> CSSKeyframeRule {
        CSSKeyframeRule {
            cssrule: CSSRule::new_inherited(parent_stylesheet),
            keyframerule: keyframerule,
            style_decl: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               keyframerule: Arc<RwLock<Keyframe>>) -> Root<CSSKeyframeRule> {
        reflect_dom_object(box CSSKeyframeRule::new_inherited(parent_stylesheet, keyframerule),
                           window,
                           CSSKeyframeRuleBinding::Wrap)
    }
}

impl CSSKeyframeRuleMethods for CSSKeyframeRule {
    // https://drafts.csswg.org/css-animations/#dom-csskeyframerule-style
    fn Style(&self) -> Root<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            CSSStyleDeclaration::new(self.global().as_window(),
                                     CSSStyleOwner::CSSRule(JS::from_ref(self.global().as_window()),
                                                                 self.keyframerule.read().block.clone()),
                                     None,
                                     CSSModificationAccess::ReadWrite)
        })
    }
}

impl SpecificCSSRule for CSSKeyframeRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::KEYFRAME_RULE
    }

    fn get_css(&self) -> DOMString {
        self.keyframerule.read().to_css_string().into()
    }
}
