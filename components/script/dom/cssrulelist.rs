/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSRuleListBinding;
use dom::bindings::codegen::Bindings::CSSRuleListBinding::CSSRuleListMethods;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::cssrule::CSSRule;
use dom::cssstylesheet::CSSStyleSheet;
use dom::window::Window;
use style::stylesheets::CssRules;

no_jsmanaged_fields!(CssRules);

#[dom_struct]
pub struct CSSRuleList {
    reflector_: Reflector,
    sheet: JS<CSSStyleSheet>,
    #[ignore_heap_size_of = "Arc"]
    rules: CssRules,
    dom_rules: Vec<MutNullableHeap<JS<CSSRule>>>
}

impl CSSRuleList {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(sheet: &CSSStyleSheet, rules: CssRules) -> CSSRuleList {
        let dom_rules = rules.0.read().iter().map(|_| MutNullableHeap::new(None)).collect();
        CSSRuleList {
            reflector_: Reflector::new(),
            sheet: JS::from_ref(sheet),
            rules: rules,
            dom_rules: dom_rules,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, sheet: &CSSStyleSheet, rules: CssRules) -> Root<CSSRuleList> {
        reflect_dom_object(box CSSRuleList::new_inherited(sheet, rules),
                           window,
                           CSSRuleListBinding::Wrap)
    }
}

impl CSSRuleListMethods for CSSRuleList {
    // https://drafts.csswg.org/cssom/#ref-for-dom-cssrulelist-item-1
    fn Item(&self, idx: u32) -> Option<Root<CSSRule>> {
        self.dom_rules.get(idx as usize).map(|rule| {
            rule.or_init(|| {
                CSSRule::new_specific(self.global().as_window(),
                                     &self.sheet,
                                     self.rules.0.read()[idx as usize].clone())
            })
        })
    }

    // https://drafts.csswg.org/cssom/#dom-cssrulelist-length
    fn Length(&self) -> u32 {
        self.dom_rules.len() as u32
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<Root<CSSRule>> {
        self.Item(index)
    }
}

