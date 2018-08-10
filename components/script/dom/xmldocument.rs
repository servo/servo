/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::XMLDocumentBinding::{self, XMLDocumentMethods};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use dom::location::Location;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use js::jsapi::JSContext;
use js::jsapi::JSObject;
use mime::Mime;
use script_traits::DocumentActivity;
use servo_url::{MutableOrigin, ServoUrl};
use std::ptr::NonNull;
use typeholder::TypeHolderTrait;

// https://dom.spec.whatwg.org/#xmldocument
#[dom_struct]
pub struct XMLDocument<TH: TypeHolderTrait> {
    document: Document<TH>,
}

impl<TH: TypeHolderTrait> XMLDocument<TH> {
    fn new_inherited(window: &Window<TH>,
                     has_browsing_context: HasBrowsingContext,
                     url: Option<ServoUrl>,
                     origin: MutableOrigin,
                     is_html_document: IsHTMLDocument,
                     content_type: Option<Mime>,
                     last_modified: Option<String>,
                     activity: DocumentActivity,
                     source: DocumentSource,
                     doc_loader: DocumentLoader) -> XMLDocument<TH> {
        XMLDocument {
            document: Document::new_inherited(window,
                                              has_browsing_context,
                                              url,
                                              origin,
                                              is_html_document,
                                              content_type,
                                              last_modified,
                                              activity,
                                              source,
                                              doc_loader,
                                              None,
                                              None,
                                              Default::default()),
        }
    }

    pub fn new(window: &Window<TH>,
               has_browsing_context: HasBrowsingContext,
               url: Option<ServoUrl>,
               origin: MutableOrigin,
               doctype: IsHTMLDocument,
               content_type: Option<Mime>,
               last_modified: Option<String>,
               activity: DocumentActivity,
               source: DocumentSource,
               doc_loader: DocumentLoader)
               -> DomRoot<XMLDocument<TH>> {
        let doc = reflect_dom_object(
            Box::new(XMLDocument::new_inherited(
                window,
                has_browsing_context,
                url,
                origin,
                doctype,
                content_type,
                last_modified,
                activity,
                source,
                doc_loader
            )),
            window,
            XMLDocumentBinding::Wrap
        );
        {
            let node = doc.upcast::<Node<TH>>();
            node.set_owner_doc(&doc.document);
        }
        doc
    }
}

impl<TH: TypeHolderTrait> XMLDocumentMethods<TH> for XMLDocument<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-document-location
    fn GetLocation(&self) -> Option<DomRoot<Location<TH>>> {
        self.upcast::<Document<TH>>().GetLocation()
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.upcast::<Document<TH>>().SupportedPropertyNames()
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter
    unsafe fn NamedGetter(&self, _cx: *mut JSContext, name: DOMString) -> Option<NonNull<JSObject>> {
        self.upcast::<Document<TH>>().NamedGetter(_cx, name)
    }
}
