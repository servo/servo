/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::HTMLDocumentBinding::HTMLDocumentMethods;
use script_bindings::error::Fallible;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::JSContext;
use script_bindings::str::DOMString;

use super::types::{Document, Location};

/// <https://html.spec.whatwg.org/multipage/#htmldocument>
#[dom_struct]
pub(crate) struct HTMLDocument {
    document: Document,
}

impl HTMLDocumentMethods<crate::DomTypeHolder> for HTMLDocument {
    /// <https://html.spec.whatwg.org/multipage/#htmldocument>
    fn GetLocation(&self) -> Option<DomRoot<Location>> {
        self.document.GetLocation()
    }

    /// <https://html.spec.whatwg.org/multipage/#htmldocument>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.document.SupportedPropertyNames()
    }

    /// <https://html.spec.whatwg.org/multipage/#htmldocument>
    fn NamedGetter(&self, _cx: JSContext, _name: DOMString) -> Fallible<Option<NonNull<JSObject>>> {
        Err(script_bindings::error::Error::NotFound)
    }
}
