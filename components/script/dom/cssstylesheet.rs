/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleSheetBinding;
use dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use dom::bindings::js::{JS, Root, MutNullableHeap};
use dom::bindings::reflector::{reflect_dom_object, Reflectable};
use dom::bindings::str::DOMString;
use dom::cssrulelist::CSSRuleList;
use dom::stylesheet::StyleSheet;
use dom::window::Window;
use std::sync::Arc;
use style::stylesheets::Stylesheet as StyleStyleSheet;

#[dom_struct]
pub struct CSSStyleSheet {
    stylesheet: StyleSheet,
    rulelist: MutNullableHeap<JS<CSSRuleList>>,
    #[ignore_heap_size_of = "Arc"]
    style_stylesheet: Arc<StyleStyleSheet>,
}

impl CSSStyleSheet {
    fn new_inherited(type_: DOMString, href: Option<DOMString>,
                     title: Option<DOMString>, stylesheet: Arc<StyleStyleSheet>) -> CSSStyleSheet {
        CSSStyleSheet {
            stylesheet: StyleSheet::new_inherited(type_, href, title),
            rulelist: MutNullableHeap::new(None),
            style_stylesheet: stylesheet,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, type_: DOMString,
               href: Option<DOMString>,
               title: Option<DOMString>,
               stylesheet: Arc<StyleStyleSheet>) -> Root<CSSStyleSheet> {
        reflect_dom_object(box CSSStyleSheet::new_inherited(type_, href, title, stylesheet),
                           window,
                           CSSStyleSheetBinding::Wrap)
    }
}

impl CSSStyleSheetMethods for CSSStyleSheet {
    // https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules
    fn CssRules(&self) -> Root<CSSRuleList> {
        self.rulelist.or_init(|| CSSRuleList::new(self.global().as_window(), self, self.style_stylesheet.rules.clone()))
    }
}

