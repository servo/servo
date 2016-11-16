/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::StyleSheetListBinding;
use dom::bindings::codegen::Bindings::StyleSheetListBinding::StyleSheetListMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::document::Document;
use dom::stylesheet::StyleSheet;
use dom::window::Window;

#[dom_struct]
pub struct StyleSheetList {
    reflector_: Reflector,
    document: JS<Document>,
}

impl StyleSheetList {
    #[allow(unrooted_must_root)]
    fn new_inherited(doc: JS<Document>) -> StyleSheetList {
        StyleSheetList {
            reflector_: Reflector::new(),
            document: doc
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, document: JS<Document>) -> Root<StyleSheetList> {
        reflect_dom_object(box StyleSheetList::new_inherited(document),
                           window, StyleSheetListBinding::Wrap)
    }
}

impl StyleSheetListMethods for StyleSheetList {
    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-length
    fn Length(&self) -> u32 {
       self.document.with_style_sheets_in_document(|s| s.len() as u32)
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-item
    fn Item(&self, index: u32) -> Option<Root<StyleSheet>> {
        // XXXManishearth this  doesn't handle the origin clean flag
        // and is a cors vulnerability
        self.document.with_style_sheets_in_document(|sheets| {
            sheets.get(index as usize)
                  .and_then(|sheet| sheet.node.get_cssom_stylesheet())
                  .map(Root::upcast)
        })
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<Root<StyleSheet>> {
        self.Item(index)
    }
}
