/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CSSStyleSheetBinding;
use dom::bindings::codegen::Bindings::CSSStyleSheetBinding::CSSStyleSheetMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::cssrulelist::CSSRuleList;
use dom::node::Node;
use dom::window::Window;
use std::sync::Arc;
use style::servo::Stylesheet;
use dom::stylesheet::StyleSheet;
use util::str::DOMString;

#[dom_struct]
pub struct CSSStyleSheet {
    ss: StyleSheet,
    stylesheet: Arc<Stylesheet>,
        //node: Node,
}

impl CSSStyleSheet {
    #[allow(unrooted_must_root)]
    fn new_inherited(stylesheet: Arc<Stylesheet>) -> CSSStyleSheet {
        CSSStyleSheet {
            ss: StyleSheet::new_inherited(DOMString::from_string(String::from("text/css")), None, None, None),
            stylesheet: stylesheet,
            //node: *node,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, stylesheet: Arc<Stylesheet>) -> Root<CSSStyleSheet> {
        reflect_dom_object(box CSSStyleSheet::new_inherited(stylesheet),
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
    // TODO: step 1
        Root::from_ref(&CSSRuleList::new_inherited(&self))
    }
}
