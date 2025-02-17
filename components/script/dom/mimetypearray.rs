/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::MimeTypeArrayBinding::MimeTypeArrayMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mimetype::MimeType;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MimeTypeArray {
    reflector_: Reflector,
}

impl MimeTypeArray {
    pub(crate) fn new_inherited() -> MimeTypeArray {
        MimeTypeArray {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<MimeTypeArray> {
        reflect_dom_object(Box::new(MimeTypeArray::new_inherited()), global, can_gc)
    }
}

impl MimeTypeArrayMethods<crate::DomTypeHolder> for MimeTypeArray {
    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-length
    fn Length(&self) -> u32 {
        0
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item
    fn Item(&self, _index: u32) -> Option<DomRoot<MimeType>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-nameditem
    fn NamedItem(&self, _name: DOMString) -> Option<DomRoot<MimeType>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item
    fn IndexedGetter(&self, _index: u32) -> Option<DomRoot<MimeType>> {
        None
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, _name: DOMString) -> Option<DomRoot<MimeType>> {
        None
    }

    // https://heycam.github.io/webidl/#dfn-supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        vec![]
    }
}
