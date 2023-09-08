/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns};
use script_traits::DocumentActivity;

use crate::document_loader::DocumentLoader;
use crate::dom::bindings::codegen::Bindings::DOMImplementationBinding::DOMImplementationMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, ElementCreationOptions,
};
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrElementCreationOptions;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::xmlname::{namespace_from_domstring, validate_qualified_name};
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::documenttype::DocumentType;
use crate::dom::htmlbodyelement::HTMLBodyElement;
use crate::dom::htmlheadelement::HTMLHeadElement;
use crate::dom::htmlhtmlelement::HTMLHtmlElement;
use crate::dom::htmltitleelement::HTMLTitleElement;
use crate::dom::node::Node;
use crate::dom::text::Text;
use crate::dom::xmldocument::XMLDocument;

// https://dom.spec.whatwg.org/#domimplementation
#[dom_struct]
pub struct DOMImplementation {
    reflector_: Reflector,
    document: Dom<Document>,
}

impl DOMImplementation {
    fn new_inherited(document: &Document) -> DOMImplementation {
        DOMImplementation {
            reflector_: Reflector::new(),
            document: Dom::from_ref(document),
        }
    }

    pub fn new(document: &Document) -> DomRoot<DOMImplementation> {
        let window = document.window();
        reflect_dom_object(Box::new(DOMImplementation::new_inherited(document)), window)
    }
}

// https://dom.spec.whatwg.org/#domimplementation
impl DOMImplementationMethods for DOMImplementation {
    // https://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    fn CreateDocumentType(
        &self,
        qualified_name: DOMString,
        pubid: DOMString,
        sysid: DOMString,
    ) -> Fallible<DomRoot<DocumentType>> {
        validate_qualified_name(&qualified_name)?;
        Ok(DocumentType::new(
            qualified_name,
            Some(pubid),
            Some(sysid),
            &self.document,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    fn CreateDocument(
        &self,
        maybe_namespace: Option<DOMString>,
        qname: DOMString,
        maybe_doctype: Option<&DocumentType>,
    ) -> Fallible<DomRoot<XMLDocument>> {
        let win = self.document.window();
        let loader = DocumentLoader::new(&self.document.loader());
        let namespace = namespace_from_domstring(maybe_namespace.to_owned());

        let content_type = match namespace {
            ns!(html) => "application/xhtml+xml".parse().unwrap(),
            ns!(svg) => mime::IMAGE_SVG,
            _ => "application/xml".parse().unwrap(),
        };

        // Step 1.
        let doc = XMLDocument::new(
            win,
            HasBrowsingContext::No,
            None,
            self.document.origin().clone(),
            IsHTMLDocument::NonHTMLDocument,
            Some(content_type),
            None,
            DocumentActivity::Inactive,
            DocumentSource::NotFromParser,
            loader,
        );
        // Step 2-3.
        let maybe_elem = if qname.is_empty() {
            None
        } else {
            let options =
                StringOrElementCreationOptions::ElementCreationOptions(ElementCreationOptions {
                    is: None,
                });
            match doc
                .upcast::<Document>()
                .CreateElementNS(maybe_namespace, qname, options)
            {
                Err(error) => return Err(error),
                Ok(elem) => Some(elem),
            }
        };

        {
            let doc_node = doc.upcast::<Node>();

            // Step 4.
            if let Some(doc_type) = maybe_doctype {
                doc_node.AppendChild(doc_type.upcast()).unwrap();
            }

            // Step 5.
            if let Some(ref elem) = maybe_elem {
                doc_node.AppendChild(elem.upcast()).unwrap();
            }
        }

        // Step 6.
        // The origin is already set

        // Step 7.
        Ok(doc)
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    fn CreateHTMLDocument(&self, title: Option<DOMString>) -> DomRoot<Document> {
        let win = self.document.window();
        let loader = DocumentLoader::new(&self.document.loader());

        // Step 1-2.
        let doc = Document::new(
            win,
            HasBrowsingContext::No,
            None,
            self.document.origin().clone(),
            IsHTMLDocument::HTMLDocument,
            None,
            None,
            DocumentActivity::Inactive,
            DocumentSource::NotFromParser,
            loader,
            None,
            None,
            Default::default(),
        );

        {
            // Step 3.
            let doc_node = doc.upcast::<Node>();
            let doc_type = DocumentType::new(DOMString::from("html"), None, None, &doc);
            doc_node.AppendChild(doc_type.upcast()).unwrap();
        }

        {
            // Step 4.
            let doc_node = doc.upcast::<Node>();
            let doc_html = DomRoot::upcast::<Node>(HTMLHtmlElement::new(
                local_name!("html"),
                None,
                &doc,
                None,
            ));
            doc_node.AppendChild(&doc_html).expect("Appending failed");

            {
                // Step 5.
                let doc_head = DomRoot::upcast::<Node>(HTMLHeadElement::new(
                    local_name!("head"),
                    None,
                    &doc,
                    None,
                ));
                doc_html.AppendChild(&doc_head).unwrap();

                // Step 6.
                if let Some(title_str) = title {
                    // Step 6.1.
                    let doc_title = DomRoot::upcast::<Node>(HTMLTitleElement::new(
                        local_name!("title"),
                        None,
                        &doc,
                        None,
                    ));
                    doc_head.AppendChild(&doc_title).unwrap();

                    // Step 6.2.
                    let title_text = Text::new(title_str, &doc);
                    doc_title.AppendChild(title_text.upcast()).unwrap();
                }
            }

            // Step 7.
            let doc_body = HTMLBodyElement::new(local_name!("body"), None, &doc, None);
            doc_html.AppendChild(doc_body.upcast()).unwrap();
        }

        // Step 8.
        // The origin is already set

        // Step 9.
        doc
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-hasfeature
    fn HasFeature(&self) -> bool {
        true
    }
}
