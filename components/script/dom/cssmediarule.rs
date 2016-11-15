/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSMediaRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::cssgroupingrule::CSSGroupingRule;
use dom::cssrule::SpecificCSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::MediaRule;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSMediaRule {
    cssrule: CSSGroupingRule,
    #[ignore_heap_size_of = "Arc"]
    mediarule: Arc<RwLock<MediaRule>>,
}

impl CSSMediaRule {
    fn new_inherited(parent: &CSSStyleSheet, mediarule: Arc<RwLock<MediaRule>>) -> CSSMediaRule {
        CSSMediaRule {
            cssrule: CSSGroupingRule::new_inherited(parent),
            mediarule: mediarule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: &CSSStyleSheet,
               mediarule: Arc<RwLock<MediaRule>>) -> Root<CSSMediaRule> {
        reflect_dom_object(box CSSMediaRule::new_inherited(parent, mediarule),
                           window,
                           CSSMediaRuleBinding::Wrap)
    }
}

impl SpecificCSSRule for CSSMediaRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::MEDIA_RULE
    }

    fn get_css(&self) -> DOMString {
        self.mediarule.read().to_css_string().into()
    }
}
