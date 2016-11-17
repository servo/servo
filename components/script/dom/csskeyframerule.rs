/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSKeyframeRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
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
}

impl CSSKeyframeRule {
    fn new_inherited(parent: Option<&CSSStyleSheet>, keyframerule: Arc<RwLock<Keyframe>>) -> CSSKeyframeRule {
        CSSKeyframeRule {
            cssrule: CSSRule::new_inherited(parent),
            keyframerule: keyframerule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: Option<&CSSStyleSheet>,
               keyframerule: Arc<RwLock<Keyframe>>) -> Root<CSSKeyframeRule> {
        reflect_dom_object(box CSSKeyframeRule::new_inherited(parent, keyframerule),
                           window,
                           CSSKeyframeRuleBinding::Wrap)
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
