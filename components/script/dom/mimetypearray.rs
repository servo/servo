/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MimeTypeArrayBinding;
use dom::bindings::codegen::Bindings::MimeTypeArrayBinding::MimeTypeArrayMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::mimetype::MimeType;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct MimeTypeArray<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
}

impl<TH: TypeHolderTrait> MimeTypeArray<TH> {
    pub fn new_inherited() -> MimeTypeArray<TH> {
        MimeTypeArray {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope<TH>) -> DomRoot<MimeTypeArray<TH>> {
        reflect_dom_object(Box::new(MimeTypeArray::new_inherited()),
                           global,
                           MimeTypeArrayBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> MimeTypeArrayMethods<TH> for MimeTypeArray<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-length
    fn Length(&self) -> u32 {
        0
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item
    fn Item(&self, _index: u32) -> Option<DomRoot<MimeType<TH>>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-nameditem
    fn NamedItem(&self, _name: DOMString) -> Option<DomRoot<MimeType<TH>>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item
    fn IndexedGetter(&self, _index: u32) -> Option<DomRoot<MimeType<TH>>> {
        None
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, _name: DOMString) -> Option<DomRoot<MimeType<TH>>> {
        None
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        vec![]
    }
}
