use dom::bindings::codegen::Bindings::StyleSheetListBinding;
use dom::bindings::codegen::Bindings::StyleSheetListBinding::StyleSheetListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::document::Document;
//use dom::stylesheet::StyleSheet;
use dom::window::Window;
//use std::cell::RefMut;
//use std::sync::Arc;
//use dom::bindings::cell::DOMRefCell;

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
                           GlobalRef::Window(window), StyleSheetListBinding::Wrap)
    }
}

impl StyleSheetListMethods for StyleSheetList {
    // 
    fn Length(&self) -> u32 {
       self.document.stylesheets().len() as u32
    }

    // 
    /*fn Item(&self, index: u32) -> Option<Ref<StyleSheet>> {
        if (index > self.Length()) {
            None
        } else {
            Some(self.document.stylesheets.borrow_mut())
        }
    }*/

}
