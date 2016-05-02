/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::ops::Deref;
use dom::bindings::codegen::Bindings::CSSStyleSheetBinding;
use dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{reflect_dom_object};
use dom::cssrulelist::CSSRuleList;
use dom::stylesheet::StyleSheet;
use dom::window::Window;
use std::sync::Arc;
use style::servo::Stylesheet;
use util::str::DOMString;

#[dom_struct]
pub struct CSSStyleSheet {
    ss: StyleSheet,
    stylesheet: Arc<Stylesheet>,
    window: JS<Window>,
}

impl CSSStyleSheet {
    #[allow(unrooted_must_root)]
    fn new_inherited(window: &Window, stylesheet: Arc<Stylesheet>) -> CSSStyleSheet {
        CSSStyleSheet {
            ss: StyleSheet::new_inherited(DOMString::from_string(String::from("text/css")), None, None, None),
            stylesheet: stylesheet,
            window: JS::from_ref(window),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, stylesheet: Arc<Stylesheet>) -> Root<CSSStyleSheet> {
        reflect_dom_object(box CSSStyleSheet::new_inherited(window, stylesheet),
                           GlobalRef::Window(window),
                           CSSStyleSheetBinding::Wrap)
    }

    pub fn get_cssstylesheet(&self) -> Arc<Stylesheet> {
        self.stylesheet.clone()
    }
}

impl CSSStyleSheetMethods for CSSStyleSheet {
    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-cssrules
    fn CssRules(&self) -> Root<CSSRuleList>  {
        Root::from_ref(&CSSRuleList::new(self.window.deref(), &self))
    }
}
