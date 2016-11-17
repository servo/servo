/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding;
use dom::bindings::codegen::Bindings::CSSKeyframesRuleBinding::CSSKeyframesRuleMethods;
use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssrulelist::{CSSRuleList, RulesSource};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::KeyframesRule;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSKeyframesRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    keyframesrule: Arc<RwLock<KeyframesRule>>,
    rulelist: MutNullableHeap<JS<CSSRuleList>>,
}

impl CSSKeyframesRule {
    fn new_inherited(parent: Option<&CSSStyleSheet>, keyframesrule: Arc<RwLock<KeyframesRule>>) -> CSSKeyframesRule {
        CSSKeyframesRule {
            cssrule: CSSRule::new_inherited(parent),
            keyframesrule: keyframesrule,
            rulelist: MutNullableHeap::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: Option<&CSSStyleSheet>,
               keyframesrule: Arc<RwLock<KeyframesRule>>) -> Root<CSSKeyframesRule> {
        reflect_dom_object(box CSSKeyframesRule::new_inherited(parent, keyframesrule),
                           window,
                           CSSKeyframesRuleBinding::Wrap)
    }

    fn rulelist(&self) -> Root<CSSRuleList> {
        self.rulelist.or_init(|| {
            let sheet = self.upcast::<CSSRule>().GetParentStyleSheet();
            let sheet = sheet.as_ref().map(|s| &**s);
            CSSRuleList::new(self.global().as_window(),
                             sheet,
                             RulesSource::Keyframes(self.keyframesrule.clone()))
        })
    }
}

impl CSSKeyframesRuleMethods for CSSKeyframesRule {
   fn CssRules(&self) -> Root<CSSRuleList> {
        self.rulelist()
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
}
