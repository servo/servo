use dom::bindings::codegen::Bindings::StyleSheetListBinding;
use dom::bindings::codegen::Bindings::StyleSheetListBinding::StyleSheetListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
//use dom::document::Document;
use dom::stylesheet::StyleSheet;
use dom::window::Window;
//use std::sync::Arc;
//use dom::bindings::cell::DOMRefCell;

#[dom_struct]
pub struct StyleSheetList {
    reflector_: Reflector,
    list: Vec<JS<StyleSheet>>,
}

impl StyleSheetList {
    #[allow(unrooted_must_root)]
    fn new_inherited(stylesheets: Vec<JS<StyleSheet>>) -> StyleSheetList {
        StyleSheetList {
            reflector_: Reflector::new(),
            list: stylesheets
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, stylesheets: Vec<JS<StyleSheet>>) -> Root<StyleSheetList> {
        reflect_dom_object(box StyleSheetList::new_inherited(stylesheets),
                           GlobalRef::Window(window), StyleSheetListBinding::Wrap)
    }
}

impl StyleSheetListMethods for StyleSheetList {
    // https://w3c.github.io/FileAPI/#dfn-length
    fn Length(&self) -> u32 {
        self.list.len() as u32
    }

    // https://w3c.github.io/FileAPI/#dfn-item
    fn Item(&self, index: u32) -> Option<Root<StyleSheet>> {
        Some(Root::from_ref(&*(self.list[index as usize])))
    }

    // check-tidy: no specs after this line
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<StyleSheet>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

