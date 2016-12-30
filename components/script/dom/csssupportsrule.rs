/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSSupportsRuleBinding;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::cssconditionrule::CSSConditionRule;
use dom::cssrule::SpecificCSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::medialist::MediaList;
use dom::window::Window;
use parking_lot::RwLock;
use std::sync::Arc;
use style::stylesheets::SupportsRule;
use style_traits::ToCss;

#[dom_struct]
pub struct CSSSupportsRule {
    cssrule: CSSConditionRule,
    #[ignore_heap_size_of = "Arc"]
    supportsrule: Arc<RwLock<SupportsRule>>,
    medialist: MutNullableJS<MediaList>,
}

impl CSSSupportsRule {
    fn new_inherited(parent_stylesheet: &CSSStyleSheet, supportsrule: Arc<RwLock<SupportsRule>>)
                     -> CSSSupportsRule {
        let list = supportsrule.read().rules.clone();
        CSSSupportsRule {
            cssrule: CSSConditionRule::new_inherited(parent_stylesheet, list),
            supportsrule: supportsrule,
            medialist: MutNullableJS::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, parent_stylesheet: &CSSStyleSheet,
               supportsrule: Arc<RwLock<SupportsRule>>) -> Root<CSSSupportsRule> {
        reflect_dom_object(box CSSSupportsRule::new_inherited(parent_stylesheet, supportsrule),
                           window,
                           CSSSupportsRuleBinding::Wrap)
    }

    fn medialist(&self) -> Root<MediaList> {
        self.medialist.or_init(|| MediaList::new(self.global().as_window(),
                                                 self.supportsrule.read().media_queries.clone()))
    }
}

impl SpecificCSSRule for CSSSupportsRule {
    fn ty(&self) -> u16 {
        use dom::bindings::codegen::Bindings::CSSRuleBinding::CSSRuleConstants;
        CSSRuleConstants::SUPPORTS_RULE
    }

    fn get_css(&self) -> DOMString {
        self.supportsrule.read().to_css_string().into()
    }
}
