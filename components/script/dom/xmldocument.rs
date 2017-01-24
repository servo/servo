/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::XMLDocumentBinding::{self, XMLDocumentMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::document::{Document, DocumentSource, IsHTMLDocument};
use dom::location::Location;
use dom::node::Node;
use dom::window::Window;
use js::jsapi::{JSContext, JSObject};
use origin::Origin;
use servo_url::ServoUrl;

// https://dom.spec.whatwg.org/#xmldocument
#[dom_struct]
pub struct XMLDocument {
    document: Document,
}

impl XMLDocument {
    fn new_inherited(window: &Window,
                     has_browsing_context: bool,
                     url: Option<ServoUrl>,
                     origin: Origin,
                     is_html_document: IsHTMLDocument,
                     content_type: Option<DOMString>,
                     last_modified: Option<String>,
                     source: DocumentSource,
                     doc_loader: DocumentLoader) -> XMLDocument {
        XMLDocument {
            document: Document::new_inherited(window,
                                              has_browsing_context,
                                              url,
                                              origin,
                                              is_html_document,
                                              content_type,
                                              last_modified,
                                              source,
                                              doc_loader,
                                              None,
                                              None),
        }
    }

    pub fn new(window: &Window,
               has_browsing_context: bool,
               url: Option<ServoUrl>,
               origin: Origin,
               doctype: IsHTMLDocument,
               content_type: Option<DOMString>,
               last_modified: Option<String>,
               source: DocumentSource,
               doc_loader: DocumentLoader)
               -> Root<XMLDocument> {
        let doc = reflect_dom_object(
            box XMLDocument::new_inherited(window,
                                           has_browsing_context,
                                           url,
                                           origin,
                                           doctype,
                                           content_type,
                                           last_modified,
                                           source,
                                           doc_loader),
            window,
            XMLDocumentBinding::Wrap);
        {
            let node = doc.upcast::<Node>();
            node.set_owner_doc(&doc.document);
        }
        doc
    }
}

impl XMLDocumentMethods for XMLDocument {
    // https://html.spec.whatwg.org/multipage/#dom-document-location
    fn GetLocation(&self) -> Option<Root<Location>> {
        self.upcast::<Document>().GetLocation()
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.upcast::<Document>().SupportedPropertyNames()
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter
    unsafe fn NamedGetter(&self, _cx: *mut JSContext, name: DOMString) -> Option<NonZero<*mut JSObject>> {
        self.upcast::<Document>().NamedGetter(_cx, name)
    }
}
