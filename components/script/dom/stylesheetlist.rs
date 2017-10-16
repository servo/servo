/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StyleSheetListBinding;
use dom::bindings::codegen::Bindings::StyleSheetListBinding::StyleSheetListMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::document::Document;
use dom::stylesheet::StyleSheet;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct StyleSheetList {
    reflector_: Reflector,
    document: Dom<Document>,
}

impl StyleSheetList {
    #[allow(unrooted_must_root)]
    fn new_inherited(doc: Dom<Document>) -> StyleSheetList {
        StyleSheetList {
            reflector_: Reflector::new(),
            document: doc
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, document: Dom<Document>) -> DomRoot<StyleSheetList> {
        reflect_dom_object(Box::new(StyleSheetList::new_inherited(document)),
                           window, StyleSheetListBinding::Wrap)
    }
}

impl StyleSheetListMethods for StyleSheetList {
    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-length
    fn Length(&self) -> u32 {
       self.document.stylesheet_count() as u32
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-item
    fn Item(&self, index: u32) -> Option<DomRoot<StyleSheet>> {
        // XXXManishearth this  doesn't handle the origin clean flag and is a
        // cors vulnerability
        self.document.stylesheet_at(index as usize).map(DomRoot::upcast)
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<StyleSheet>> {
        self.Item(index)
    }
}
