use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLUListElement {
    parent: HTMLElement
}

impl HTMLUListElement {
    pub fn Compact(&self) -> bool {
        false
    }
    
    pub fn SetCompact(&mut self, _compact: bool, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        null_string
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }
}
