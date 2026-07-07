/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::{JSContext, NoGC};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::MimeTypeArrayBinding::MimeTypeArrayMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mimetype::MimeType;

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

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<MimeTypeArray> {
        reflect_dom_object_with_cx(Box::new(MimeTypeArray::new_inherited()), global, cx)
    }
}

impl MimeTypeArrayMethods<crate::DomTypeHolder> for MimeTypeArray {
    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-length>
    fn Length(&self) -> u32 {
        0
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item>
    fn Item(&self, _index: u32) -> Option<DomRoot<MimeType>> {
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-nameditem>
    fn NamedItem(&self, _name: DOMString) -> Option<DomRoot<MimeType>> {
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item>
    fn IndexedGetter(&self, _index: u32) -> Option<DomRoot<MimeType>> {
        None
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, _name: DOMString) -> Option<DomRoot<MimeType>> {
        None
    }

    /// <https://heycam.github.io/webidl/#dfn-supported-property-names>
    fn SupportedPropertyNames(&self, _: &NoGC) -> Vec<DOMString> {
        vec![]
    }
}
