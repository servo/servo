/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleSheetBinding;
use dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::reflector::{reflect_dom_object, DomObject};
use dom::bindings::str::DOMString;
use dom::cssrulelist::{CSSRuleList, RulesSource};
use dom::element::Element;
use dom::stylesheet::StyleSheet;
use dom::window::Window;
use std::sync::Arc;
use style::stylesheets::Stylesheet as StyleStyleSheet;

#[dom_struct]
pub struct CSSStyleSheet {
    stylesheet: StyleSheet,
    owner: JS<Element>,
    rulelist: MutNullableJS<CSSRuleList>,
    #[ignore_heap_size_of = "Arc"]
    style_stylesheet: Arc<StyleStyleSheet>,
}

impl CSSStyleSheet {
    fn new_inherited(owner: &Element,
                     type_: DOMString,
                     href: Option<DOMString>,
                     title: Option<DOMString>,
                     stylesheet: Arc<StyleStyleSheet>) -> CSSStyleSheet {
        CSSStyleSheet {
            stylesheet: StyleSheet::new_inherited(type_, href, title),
            owner: JS::from_ref(owner),
            rulelist: MutNullableJS::new(None),
            style_stylesheet: stylesheet,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window,
               owner: &Element,
               type_: DOMString,
               href: Option<DOMString>,
               title: Option<DOMString>,
               stylesheet: Arc<StyleStyleSheet>) -> Root<CSSStyleSheet> {
        reflect_dom_object(box CSSStyleSheet::new_inherited(owner, type_, href, title, stylesheet),
                           window,
                           CSSStyleSheetBinding::Wrap)
    }

    fn rulelist(&self) -> Root<CSSRuleList> {
        self.rulelist.or_init(|| CSSRuleList::new(self.global().as_window(),
                                                  self,
                                                  RulesSource::Rules(self.style_stylesheet
                                                                         .rules.clone())))
    }

    pub fn disabled(&self) -> bool {
        self.style_stylesheet.disabled()
    }

    pub fn set_disabled(&self, disabled: bool) {
        if self.style_stylesheet.set_disabled(disabled) {
            self.global().as_window().Document().invalidate_stylesheets();
        }
    }

    pub fn style_stylesheet(&self) -> &StyleStyleSheet {
        &self.style_stylesheet
    }
}

impl CSSStyleSheetMethods for CSSStyleSheet {
    // https://drafts.csswg.org/cssom/#dom-cssstylesheet-cssrules
    fn CssRules(&self) -> Root<CSSRuleList> {
        // XXXManishearth check origin clean flag
        // https://github.com/servo/servo/issues/14327
        self.rulelist()
    }

    // https://drafts.csswg.org/cssom/#dom-cssstylesheet-insertrule
    fn InsertRule(&self, rule: DOMString, index: u32) -> Fallible<u32> {
        // XXXManishearth check origin clean flag
        self.rulelist().insert_rule(&rule, index, /* nested */ false)
    }

    // https://drafts.csswg.org/cssom/#dom-cssstylesheet-deleterule
    fn DeleteRule(&self, index: u32) -> ErrorResult {
        // XXXManishearth check origin clean flag
        self.rulelist().remove_rule(index)
    }
}

