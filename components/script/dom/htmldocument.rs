/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::jsapi::JSObject;

use crate::dom::bindings::codegen::Bindings::HTMLDocumentBinding::HTMLDocumentMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::location::Location;
use crate::script_runtime::JSContext as SafeJSContext;

// https://html.spec.whatwg.org/multipage/#htmldocument
#[dom_struct]
pub struct HTMLDocument {
    reflector_: Reflector,
}

impl HTMLDocumentMethods for HTMLDocument {
    // https://html.spec.whatwg.org/multipage/#htmldocument
    fn GetLocation(&self) -> Option<DomRoot<Location>> {
        None
    }

    // https://html.spec.whatwg.org/multipage/#htmldocument
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        Vec::new()
    }

    // https://html.spec.whatwg.org/multipage/#htmldocument
    fn NamedGetter(
        &self,
        _cx: SafeJSContext,
        _name: DOMString,
    ) -> Fallible<Option<NonNull<JSObject>>> {
        Err(Error::NotFound)
    }
}
