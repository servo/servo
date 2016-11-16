/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StyleSheetBinding;
use dom::bindings::codegen::Bindings::StyleSheetBinding::StyleSheetMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::window::Window;


#[dom_struct]
pub struct StyleSheet {
    reflector_: Reflector,
    type_: DOMString,
    href: Option<DOMString>,
    title: Option<DOMString>,
}

impl StyleSheet {
    #[allow(unrooted_must_root)]
    pub fn new_inherited(type_: DOMString, href: Option<DOMString>, title: Option<DOMString>) -> StyleSheet {
        StyleSheet {
            reflector_: Reflector::new(),
            type_: type_,
            href: href,
            title: title
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, type_: DOMString,
               href: Option<DOMString>,
               title: Option<DOMString>) -> Root<StyleSheet> {
        reflect_dom_object(box StyleSheet::new_inherited(type_, href, title),
                           window,
                           StyleSheetBinding::Wrap)
    }
}


impl StyleSheetMethods for StyleSheet {
    // https://drafts.csswg.org/cssom/#dom-stylesheet-type
    fn Type_(&self) -> DOMString {
        self.type_.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-href
    fn GetHref(&self) -> Option<DOMString> {
        self.href.clone()
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheet-title
    fn GetTitle(&self) -> Option<DOMString> {
        self.title.clone()
    }
}

