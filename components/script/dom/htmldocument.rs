/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::HTMLDocumentBinding::HTMLDocumentMethods;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use script_bindings::str::DOMString;

use super::types::{Document, Location};
use crate::dom::bindings::codegen::Bindings::DocumentBinding::NamedPropertyValue;

/// <https://html.spec.whatwg.org/multipage/#htmldocument>
#[dom_struct]
pub(crate) struct HTMLDocument {
    document: Document,
}

impl HTMLDocumentMethods<crate::DomTypeHolder> for HTMLDocument {
    /// <https://html.spec.whatwg.org/multipage/#dom-document-location>
    fn GetLocation(&self) -> Option<DomRoot<Location>> {
        self.document.GetLocation()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.document.SupportedPropertyNames()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter>
    fn NamedGetter(&self, name: DOMString) -> Option<NamedPropertyValue> {
        self.document.NamedGetter(name, CanGc::note())
    }
}
