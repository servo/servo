/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::Bindings::XMLDocumentBinding::{self, XMLDocumentMethods};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::document::{Document, DocumentSource, IsHTMLDocument};
use dom::node::Node;
use dom::window::Window;
use js::jsapi::{JSContext, JSObject};
use url::Url;
use util::str::DOMString;

// https://dom.spec.whatwg.org/#xmldocument
#[dom_struct]
pub struct XMLDocument {
    document: Document,
}

impl XMLDocument {
    fn new_inherited(window: &Window,
                     url: Option<Url>,
                     is_html_document: IsHTMLDocument,
                     content_type: Option<DOMString>,
                     last_modified: Option<String>,
                     source: DocumentSource,
                     doc_loader: DocumentLoader) -> XMLDocument {
        XMLDocument {
            document: Document::new_inherited(window,
                                              url,
                                              is_html_document,
                                              content_type,
                                              last_modified,
                                              source,
                                              doc_loader),
        }
    }

    pub fn new(window: &Window,
               url: Option<Url>,
               doctype: IsHTMLDocument,
               content_type: Option<DOMString>,
               last_modified: Option<String>,
               source: DocumentSource,
               doc_loader: DocumentLoader)
               -> Root<XMLDocument> {
        let doc = reflect_dom_object(
            box XMLDocument::new_inherited(window,
                                           url,
                                           doctype,
                                           content_type,
                                           last_modified,
                                           source,
                                           doc_loader),
            GlobalRef::Window(window),
            XMLDocumentBinding::Wrap);
        {
            let node = doc.upcast::<Node>();
            node.set_owner_doc(&doc.r().document);
        }
        doc
    }

    pub fn Constructor(global: GlobalRef) -> Fallible<Root<XMLDocument>> {
        let win = global.as_window();
        let doc = win.Document();
        let doc = doc.r();
        let docloader = DocumentLoader::new(&*doc.loader());

        Ok(XMLDocument::new(win,
                            None,
                            IsHTMLDocument::NonHTMLDocument,
                            None,
                            None,
                            DocumentSource::NotFromParser,
                            docloader))
    }
}

impl XMLDocumentMethods for XMLDocument {
    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.document.SupportedPropertyNames()
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter
    fn NamedGetter(&self, _cx: *mut JSContext, name: DOMString, found: &mut bool) -> *mut JSObject {
        self.document.NamedGetter(_cx, name, found)
    }
}
