/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSViewportRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::cssrule::{CSSRule, SpecificCSSRule};
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::viewport::ViewportRule;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSViewportRule {
    cssrule: CSSRule,
    #[ignore_heap_size_of = "Arc"]
    viewportrule: Arc<RwLock<ViewportRule>>,
}

impl CSSViewportRule {
    fn new_inherited(parent: &CSSStyleSheet, viewportrule: Arc<RwLock<ViewportRule>>) -> CSSViewportRule {
        CSSViewportRule {
            cssrule: CSSRule::new_inherited(parent),
            viewportrule: viewportrule,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: &CSSStyleSheet,
               viewportrule: Arc<RwLock<ViewportRule>>) -> Root<CSSViewportRule> {
        reflect_dom_object(box CSSViewportRule::new_inherited(parent, viewportrule),
                           window,
                           CSSViewportRuleBinding::Wrap)
    }
}

impl SpecificCSSRule for CSSViewportRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::VIEWPORT_RULE
    }

    fn get_css(&self) -> DOMString {
        self.viewportrule.read().to_css_string().into()
    }
}
