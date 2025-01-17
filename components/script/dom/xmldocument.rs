/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use mime::Mime;
use net_traits::request::InsecureRequestsPolicy;
use script_traits::DocumentActivity;
use servo_url::{MutableOrigin, ServoUrl};

use crate::document_loader::DocumentLoader;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, NamedPropertyValue,
};
use crate::dom::bindings::codegen::Bindings::XMLDocumentBinding::XMLDocumentMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::location::Location;
use crate::dom::node::Node;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

// https://dom.spec.whatwg.org/#xmldocument
#[dom_struct]
pub(crate) struct XMLDocument {
    document: Document,
}

impl XMLDocument {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        window: &Window,
        has_browsing_context: HasBrowsingContext,
        url: Option<ServoUrl>,
        origin: MutableOrigin,
        is_html_document: IsHTMLDocument,
        content_type: Option<Mime>,
        last_modified: Option<String>,
        activity: DocumentActivity,
        source: DocumentSource,
        doc_loader: DocumentLoader,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
    ) -> XMLDocument {
        XMLDocument {
            document: Document::new_inherited(
                window,
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
                Default::default(),
                false,
                inherited_insecure_requests_policy,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        has_browsing_context: HasBrowsingContext,
        url: Option<ServoUrl>,
        origin: MutableOrigin,
        doctype: IsHTMLDocument,
        content_type: Option<Mime>,
        last_modified: Option<String>,
        activity: DocumentActivity,
        source: DocumentSource,
        doc_loader: DocumentLoader,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
    ) -> DomRoot<XMLDocument> {
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
                doc_loader,
                inherited_insecure_requests_policy,
            )),
            window,
            CanGc::note(),
        );
        {
            let node = doc.upcast::<Node>();
            node.set_owner_doc(&doc.document);
        }
        doc
    }
}

impl XMLDocumentMethods<crate::DomTypeHolder> for XMLDocument {
    // https://html.spec.whatwg.org/multipage/#dom-document-location
    fn GetLocation(&self) -> Option<DomRoot<Location>> {
        self.upcast::<Document>().GetLocation()
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:supported-property-names
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.upcast::<Document>().SupportedPropertyNames()
    }

    // https://html.spec.whatwg.org/multipage/#dom-tree-accessors:dom-document-nameditem-filter
    fn NamedGetter(&self, name: DOMString) -> Option<NamedPropertyValue> {
        self.upcast::<Document>().NamedGetter(name)
    }
}
