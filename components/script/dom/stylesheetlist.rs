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
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct StyleSheetList<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    document: Dom<Document<TH>>,
}

impl<TH: TypeHolderTrait> StyleSheetList<TH> {
    #[allow(unrooted_must_root)]
    fn new_inherited(doc: Dom<Document<TH>>) -> StyleSheetList<TH> {
        StyleSheetList {
            reflector_: Reflector::new(),
            document: doc
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window<TH>, document: Dom<Document<TH>>) -> DomRoot<StyleSheetList<TH>> {
        reflect_dom_object(Box::new(StyleSheetList::new_inherited(document)),
                           window, StyleSheetListBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> StyleSheetListMethods<TH> for StyleSheetList<TH> {
    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-length
    fn Length(&self) -> u32 {
       self.document.stylesheet_count() as u32
    }

    // https://drafts.csswg.org/cssom/#dom-stylesheetlist-item
    fn Item(&self, index: u32) -> Option<DomRoot<StyleSheet<TH>>> {
        // XXXManishearth this  doesn't handle the origin clean flag and is a
        // cors vulnerability
        self.document.stylesheet_at(index as usize).map(DomRoot::upcast)
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<StyleSheet<TH>>> {
        self.Item(index)
    }
}
