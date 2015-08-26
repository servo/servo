/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::DocumentLoader;
use dom::bindings::codegen::Bindings::DOMImplementationBinding;
use dom::bindings::codegen::Bindings::DOMImplementationBinding::DOMImplementationMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::validate_qualified_name;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::document::DocumentSource;
use dom::document::{Document, IsHTMLDocument};
use dom::documenttype::DocumentType;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::text::Text;
use util::str::DOMString;

use std::borrow::ToOwned;

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
                           GlobalRef::Window(window.r()),
                           DOMImplementationBinding::Wrap)
    }
}

// https://dom.spec.whatwg.org/#domimplementation
impl<'a> DOMImplementationMethods for &'a DOMImplementation {
    // https://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    fn CreateDocumentType(self, qualified_name: DOMString, pubid: DOMString, sysid: DOMString)
                          -> Fallible<Root<DocumentType>> {
        try!(validate_qualified_name(&qualified_name));
        let document = self.document.root();
        Ok(DocumentType::new(qualified_name, Some(pubid), Some(sysid), document.r()))
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    fn CreateDocument(self, namespace: Option<DOMString>, qname: DOMString,
                      maybe_doctype: Option<&DocumentType>) -> Fallible<Root<Document>> {
        let doc = self.document.root();
        let doc = doc.r();
        let win = doc.window();
        let loader = DocumentLoader::new(&*doc.loader());

        // Step 1.
        let doc = Document::new(win.r(), None, IsHTMLDocument::NonHTMLDocument,
                                None, None, DocumentSource::NotFromParser, loader);
        // Step 2-3.
        let maybe_elem = if qname.is_empty() {
            None
        } else {
            match doc.r().CreateElementNS(namespace, qname) {
                Err(error) => return Err(error),
                Ok(elem) => Some(elem)
            }
        };

        {
            let doc_node = NodeCast::from_ref(doc.r());

            // Step 4.
            match maybe_doctype {
                None => (),
                Some(ref doctype) => {
                    let doc_type = NodeCast::from_ref(*doctype);
                    assert!(doc_node.AppendChild(doc_type).is_ok())
                }
            }

            // Step 5.
            match maybe_elem {
                None => (),
                Some(ref elem) => {
                    assert!(doc_node.AppendChild(NodeCast::from_ref(elem.r())).is_ok())
                }
            }
        }

        // Step 6.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 7.
        Ok(doc)
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    fn CreateHTMLDocument(self, title: Option<DOMString>) -> Root<Document> {
        let document = self.document.root();
        let document = document.r();
        let win = document.window();
        let loader = DocumentLoader::new(&*document.loader());

        // Step 1-2.
        let doc = Document::new(win.r(), None, IsHTMLDocument::HTMLDocument, None, None,
                                DocumentSource::NotFromParser, loader);

        {
            // Step 3.
            let doc_node = NodeCast::from_ref(doc.r());
            let doc_type = DocumentType::new("html".to_owned(), None, None, doc.r());
            assert!(doc_node.AppendChild(NodeCast::from_ref(doc_type.r())).is_ok());
        }

        {
            // Step 4.
            let doc_node = NodeCast::from_ref(doc.r());
            let doc_html = NodeCast::from_root(
                HTMLHtmlElement::new("html".to_owned(), None, doc.r()));
            assert!(doc_node.AppendChild(doc_html.r()).is_ok());

            {
                // Step 5.
                let doc_head = NodeCast::from_root(
                    HTMLHeadElement::new("head".to_owned(), None, doc.r()));
                assert!(doc_html.r().AppendChild(doc_head.r()).is_ok());

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let doc_title = NodeCast::from_root(
                            HTMLTitleElement::new("title".to_owned(), None, doc.r()));
                        assert!(doc_head.r().AppendChild(doc_title.r()).is_ok());

                        // Step 6.2.
                        let title_text = Text::new(title_str, doc.r());
                        assert!(doc_title.r().AppendChild(NodeCast::from_ref(title_text.r())).is_ok());
                    }
                }
            }

            // Step 7.
            let doc_body = HTMLBodyElement::new("body".to_owned(), None, doc.r());
            assert!(doc_html.r().AppendChild(NodeCast::from_ref(doc_body.r())).is_ok());
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        doc
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-hasfeature
    fn HasFeature(self) -> bool {
        true
    }
}
