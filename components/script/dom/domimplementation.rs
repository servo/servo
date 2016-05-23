/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DOMImplementationBinding;
use dom::bindings::codegen::Bindings::DOMImplementationBinding::DOMImplementationMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bindings::xmlname::{validate_qualified_name, namespace_from_domstring};
use dom::document::DocumentSource;
use dom::document::{Document, IsHTMLDocument};
use dom::documenttype::DocumentType;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::Node;
use dom::text::Text;
use dom::xmldocument::XMLDocument;

// https://dom.spec.whatwg.org/#domimplementation
#[dom_struct]
pub struct DOMImplementation {
    reflector_: Reflector,
    document: JS<Document>,
}

impl DOMImplementation {
    fn new_inherited(document: &Document) -> DOMImplementation {
        DOMImplementation {
            reflector_: Reflector::new(),
            document: JS::from_ref(document),
        }
    }

    pub fn new(document: &Document) -> Root<DOMImplementation> {
        let window = document.window();
        reflect_dom_object(box DOMImplementation::new_inherited(document),
                           GlobalRef::Window(window),
                           DOMImplementationBinding::Wrap)
    }
}

// https://dom.spec.whatwg.org/#domimplementation
impl DOMImplementationMethods for DOMImplementation {
    // https://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    fn CreateDocumentType(&self,
                          qualified_name: DOMString,
                          pubid: DOMString,
                          sysid: DOMString)
                          -> Fallible<Root<DocumentType>> {
        try!(validate_qualified_name(&qualified_name));
        Ok(DocumentType::new(qualified_name, Some(pubid), Some(sysid), &self.document))
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    fn CreateDocument(&self,
                      maybe_namespace: Option<DOMString>,
                      qname: DOMString,
                      maybe_doctype: Option<&DocumentType>)
                      -> Fallible<Root<XMLDocument>> {
        let win = self.document.window();
        let loader = DocumentLoader::new(&self.document.loader());
        let namespace = namespace_from_domstring(maybe_namespace.to_owned());

        let content_type = match namespace {
            ns!(html) => "application/xhtml+xml",
            ns!(svg) => "image/svg+xml",
            _ => "application/xml"
        };

        // Step 1.
        let doc = XMLDocument::new(win,
                                   None,
                                   None,
                                   IsHTMLDocument::NonHTMLDocument,
                                   Some(DOMString::from(content_type)),
                                   None,
                                   DocumentSource::NotFromParser,
                                   loader);
        // Step 2-3.
        let maybe_elem = if qname.is_empty() {
            None
        } else {
            match doc.upcast::<Document>().CreateElementNS(maybe_namespace, qname) {
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
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 7.
        Ok(doc)
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    fn CreateHTMLDocument(&self, title: Option<DOMString>) -> Root<Document> {
        let win = self.document.window();
        let loader = DocumentLoader::new(&self.document.loader());

        // Step 1-2.
        let doc = Document::new(win,
                                None,
                                None,
                                IsHTMLDocument::HTMLDocument,
                                None,
                                None,
                                DocumentSource::NotFromParser,
                                loader);

        {
            // Step 3.
            let doc_node = doc.upcast::<Node>();
            let doc_type = DocumentType::new(DOMString::from("html"), None, None, doc.r());
            doc_node.AppendChild(doc_type.upcast()).unwrap();
        }

        {
            // Step 4.
            let doc_node = doc.upcast::<Node>();
            let doc_html = Root::upcast::<Node>(HTMLHtmlElement::new(atom!("html"),
                                                                     None,
                                                                     doc.r()));
            doc_node.AppendChild(&doc_html).expect("Appending failed");

            {
                // Step 5.
                let doc_head = Root::upcast::<Node>(HTMLHeadElement::new(atom!("head"),
                                                                         None,
                                                                         doc.r()));
                doc_html.AppendChild(&doc_head).unwrap();

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let doc_title =
                            Root::upcast::<Node>(HTMLTitleElement::new(atom!("title"),
                                                                       None,
                                                                       doc.r()));
                        doc_head.AppendChild(&doc_title).unwrap();

                        // Step 6.2.
                        let title_text = Text::new(title_str, doc.r());
                        doc_title.AppendChild(title_text.upcast()).unwrap();
                    }
                }
            }

            // Step 7.
            let doc_body = HTMLBodyElement::new(atom!("body"), None, doc.r());
            doc_html.AppendChild(doc_body.upcast()).unwrap();
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        doc
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-hasfeature
    fn HasFeature(&self) -> bool {
        true
    }
}
