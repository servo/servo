/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSGroupingRuleBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::cssrule::CSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;

#[dom_struct]
pub struct CSSGroupingRule {
    cssrule: CSSRule,
}

impl CSSGroupingRule {
    pub fn new_inherited(parent: &CSSStyleSheet) -> CSSGroupingRule {
        CSSGroupingRule {
            cssrule: CSSRule::new_inherited(parent),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent: &CSSStyleSheet) -> Root<CSSGroupingRule> {
        reflect_dom_object(box CSSGroupingRule::new_inherited(parent),
                           window,
                           CSSGroupingRuleBinding::Wrap)
    }
}
