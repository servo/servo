/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{QualName, local_name, ns};
use script_bindings::error::Error;
use script_traits::DocumentActivity;

use crate::document_loader::DocumentLoader;
use crate::dom::bindings::codegen::Bindings::DOMImplementationBinding::DOMImplementationMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::{
    DocumentMethods, ElementCreationOptions,
};
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrElementCreationOptions;
use crate::dom::bindings::domname::{is_valid_doctype_name, namespace_from_domstring};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::documenttype::DocumentType;
use crate::dom::element::{CustomElementCreationMode, ElementCreator};
use crate::dom::node::Node;
use crate::dom::text::Text;
use crate::dom::types::Element;
use crate::dom::xmldocument::XMLDocument;
use crate::script_runtime::CanGc;

// https://dom.spec.whatwg.org/#domimplementation
#[dom_struct]
pub(crate) struct DOMImplementation {
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

    pub(crate) fn new(document: &Document, can_gc: CanGc) -> DomRoot<DOMImplementation> {
        let window = document.window();
        reflect_dom_object(
            Box::new(DOMImplementation::new_inherited(document)),
            window,
            can_gc,
        )
    }
}

// https://dom.spec.whatwg.org/#domimplementation
impl DOMImplementationMethods<crate::DomTypeHolder> for DOMImplementation {
    /// <https://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype>
    fn CreateDocumentType(
        &self,
        qualified_name: DOMString,
        pubid: DOMString,
        sysid: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DocumentType>> {
        // Step 1. If name is not a valid doctype name, then throw an
        //      "InvalidCharacterError" DOMException.
        if !is_valid_doctype_name(&qualified_name) {
            debug!("Not a valid doctype name");
            return Err(Error::InvalidCharacter(None));
        }

        Ok(DocumentType::new(
            qualified_name,
            Some(pubid),
            Some(sysid),
            &self.document,
            can_gc,
        ))
    }

    /// <https://dom.spec.whatwg.org/#dom-domimplementation-createdocument>
    fn CreateDocument(
        &self,
        maybe_namespace: Option<DOMString>,
        qname: DOMString,
        maybe_doctype: Option<&DocumentType>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<XMLDocument>> {
        let win = self.document.window();
        let loader = DocumentLoader::new(&self.document.loader());
        let namespace = namespace_from_domstring(maybe_namespace.to_owned());

        let content_type = match namespace {
            ns!(html) => "application/xhtml+xml",
            ns!(svg) => "image/svg+xml",
            _ => "application/xml",
        }
        .parse()
        .unwrap();

        // Step 1. Let document be a new XMLDocument.
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
            Some(self.document.insecure_requests_policy()),
            self.document.has_trustworthy_ancestor_or_current_origin(),
            self.document.custom_element_reaction_stack(),
            can_gc,
        );

        // Step 2. Let element be null.
        // Step 3. If qualifiedName is not the empty string, then set element to the result of running
        // the internal createElementNS steps, given document, namespace, qualifiedName, and an empty dictionary.
        let maybe_elem = if qname.is_empty() {
            None
        } else {
            let options =
                StringOrElementCreationOptions::ElementCreationOptions(ElementCreationOptions {
                    is: None,
                });
            match doc
                .upcast::<Document>()
                .CreateElementNS(maybe_namespace, qname, options, can_gc)
            {
                Err(error) => return Err(error),
                Ok(elem) => Some(elem),
            }
        };

        {
            let doc_node = doc.upcast::<Node>();

            // Step 4.
            if let Some(doc_type) = maybe_doctype {
                doc_node.AppendChild(doc_type.upcast(), can_gc).unwrap();
            }

            // Step 5.
            if let Some(ref elem) = maybe_elem {
                doc_node.AppendChild(elem.upcast(), can_gc).unwrap();
            }
        }

        // Step 6.
        // The origin is already set

        // Step 7.
        Ok(doc)
    }

    /// <https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument>
    fn CreateHTMLDocument(&self, title: Option<DOMString>, can_gc: CanGc) -> DomRoot<Document> {
        let win = self.document.window();
        let loader = DocumentLoader::new(&self.document.loader());

        // Step 1. Let doc be a new document that is an HTML document.
        // Step 2. Set doc’s content type to "text/html".
        let doc = Document::new(
            win,
            HasBrowsingContext::No,
            None,
            None,
            // Step 8. doc’s origin is this’s associated document’s origin.
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
            false,
            self.document.allow_declarative_shadow_roots(),
            Some(self.document.insecure_requests_policy()),
            self.document.has_trustworthy_ancestor_or_current_origin(),
            self.document.custom_element_reaction_stack(),
            self.document.creation_sandboxing_flag_set(),
            can_gc,
        );

        {
            // Step 3. Append a new doctype, with "html" as its name and with its node document set to doc, to doc.
            let doc_node = doc.upcast::<Node>();
            let doc_type = DocumentType::new(DOMString::from("html"), None, None, &doc, can_gc);
            doc_node.AppendChild(doc_type.upcast(), can_gc).unwrap();
        }

        {
            // Step 4. Append the result of creating an element given doc, "html",
            // and the HTML namespace, to doc.
            let doc_node = doc.upcast::<Node>();
            let doc_html = DomRoot::upcast::<Node>(Element::create(
                QualName::new(None, ns!(html), local_name!("html")),
                None,
                &doc,
                ElementCreator::ScriptCreated,
                CustomElementCreationMode::Asynchronous,
                None,
                can_gc,
            ));
            doc_node
                .AppendChild(&doc_html, can_gc)
                .expect("Appending failed");

            {
                // Step 5. Append the result of creating an element given doc, "head",
                // and the HTML namespace, to the html element created earlier.
                let doc_head = DomRoot::upcast::<Node>(Element::create(
                    QualName::new(None, ns!(html), local_name!("head")),
                    None,
                    &doc,
                    ElementCreator::ScriptCreated,
                    CustomElementCreationMode::Asynchronous,
                    None,
                    can_gc,
                ));
                doc_html.AppendChild(&doc_head, can_gc).unwrap();

                // Step 6. If title is given:
                if let Some(title_str) = title {
                    // Step 6.1. Append the result of creating an element given doc, "title",
                    // and the HTML namespace, to the head element created earlier.
                    let doc_title = DomRoot::upcast::<Node>(Element::create(
                        QualName::new(None, ns!(html), local_name!("title")),
                        None,
                        &doc,
                        ElementCreator::ScriptCreated,
                        CustomElementCreationMode::Asynchronous,
                        None,
                        can_gc,
                    ));
                    doc_head.AppendChild(&doc_title, can_gc).unwrap();

                    // Step 6.2. Append a new Text node, with its data set to title (which could be the empty string)
                    // and its node document set to doc, to the title element created earlier.
                    let title_text = Text::new(title_str, &doc, can_gc);
                    doc_title.AppendChild(title_text.upcast(), can_gc).unwrap();
                }
            }

            // Step 7. Append the result of creating an element given doc, "body",
            // and the HTML namespace, to the html element created earlier.
            let doc_body = Element::create(
                QualName::new(None, ns!(html), local_name!("body")),
                None,
                &doc,
                ElementCreator::ScriptCreated,
                CustomElementCreationMode::Asynchronous,
                None,
                can_gc,
            );
            doc_html.AppendChild(doc_body.upcast(), can_gc).unwrap();
        }

        // Step 9. Return doc.
        doc
    }

    /// <https://dom.spec.whatwg.org/#dom-domimplementation-hasfeature>
    fn HasFeature(&self) -> bool {
        true
    }
}
