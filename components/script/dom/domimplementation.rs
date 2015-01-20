/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::DOMImplementationBinding;
use dom::bindings::codegen::Bindings::DOMImplementationBinding::DOMImplementationMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::Fallible;
use dom::bindings::error::Error::{InvalidCharacter, NamespaceError};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, Root, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::bindings::utils::xml_name_type;
use dom::bindings::utils::XMLName::{QName, Name, InvalidXMLName};
use dom::document::{Document, DocumentHelpers, IsHTMLDocument};
use dom::document::DocumentSource;
use dom::documenttype::DocumentType;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::Node;
use dom::text::Text;
use servo_util::str::DOMString;

use std::borrow::ToOwned;

#[dom_struct]
pub struct DOMImplementation {
    reflector_: Reflector,
    document: JS<Document>,
}

impl DOMImplementation {
    fn new_inherited(document: JSRef<Document>) -> DOMImplementation {
        DOMImplementation {
            reflector_: Reflector::new(),
            document: JS::from_rooted(document),
        }
    }

    pub fn new(document: JSRef<Document>) -> Temporary<DOMImplementation> {
        let window = document.window().root();
        reflect_dom_object(box DOMImplementation::new_inherited(document),
                           GlobalRef::Window(window.r()),
                           DOMImplementationBinding::Wrap)
    }
}

// http://dom.spec.whatwg.org/#domimplementation
impl<'a> DOMImplementationMethods for JSRef<'a, DOMImplementation> {
    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    fn CreateDocumentType(self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<Temporary<DocumentType>> {
        match xml_name_type(qname.as_slice()) {
            // Step 1.
            InvalidXMLName => Err(InvalidCharacter),
            // Step 2.
            Name => Err(NamespaceError),
            // Step 3.
            QName => {
                let document = self.document.root();
                Ok(DocumentType::new(qname, Some(pubid), Some(sysid), document.r()))
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    fn CreateDocument(self, namespace: Option<DOMString>, qname: DOMString,
                      maybe_doctype: Option<JSRef<DocumentType>>) -> Fallible<Temporary<Document>> {
        let doc = self.document.root();
        let win = doc.r().window().root();

        // Step 1.
        let doc = Document::new(win.r(), None, IsHTMLDocument::NonHTMLDocument,
                                None, DocumentSource::NotFromParser).root();
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
            let doc_node: JSRef<Node> = NodeCast::from_ref(doc.r());

            // Step 4.
            match maybe_doctype {
                None => (),
                Some(ref doctype) => {
                    let doc_type: JSRef<Node> = NodeCast::from_ref(*doctype);
                    assert!(doc_node.AppendChild(doc_type).is_ok())
                }
            }

            // Step 5.
            match maybe_elem.root() {
                None => (),
                Some(elem) => {
                    assert!(doc_node.AppendChild(NodeCast::from_ref(elem.r())).is_ok())
                }
            }
        }

        // Step 6.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 7.
        Ok(Temporary::from_rooted(doc.r()))
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    fn CreateHTMLDocument(self, title: Option<DOMString>) -> Temporary<Document> {
        let document = self.document.root();
        let win = document.r().window().root();

        // Step 1-2.
        let doc = Document::new(win.r(), None, IsHTMLDocument::HTMLDocument, None,
                                DocumentSource::NotFromParser).root();
        let doc_node: JSRef<Node> = NodeCast::from_ref(doc.r());

        {
            // Step 3.
            let doc_type = DocumentType::new("html".to_owned(), None, None, doc.r()).root();
            assert!(doc_node.AppendChild(NodeCast::from_ref(doc_type.r())).is_ok());
        }

        {
            // Step 4.
            let doc_html: Root<Node> = NodeCast::from_temporary(HTMLHtmlElement::new("html".to_owned(), None, doc.r())).root();
            assert!(doc_node.AppendChild(doc_html.r()).is_ok());

            {
                // Step 5.
                let doc_head: Root<Node> = NodeCast::from_temporary(HTMLHeadElement::new("head".to_owned(), None, doc.r())).root();
                assert!(doc_html.r().AppendChild(doc_head.r()).is_ok());

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let doc_title: Root<Node> = NodeCast::from_temporary(HTMLTitleElement::new("title".to_owned(), None, doc.r())).root();
                        assert!(doc_head.r().AppendChild(doc_title.r()).is_ok());

                        // Step 6.2.
                        let title_text: Root<Text> = Text::new(title_str, doc.r()).root();
                        assert!(doc_title.r().AppendChild(NodeCast::from_ref(title_text.r())).is_ok());
                    }
                }
            }

            // Step 7.
            let doc_body: Root<HTMLBodyElement> = HTMLBodyElement::new("body".to_owned(), None, doc.r()).root();
            assert!(doc_html.r().AppendChild(NodeCast::from_ref(doc_body.r())).is_ok());
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        Temporary::from_rooted(doc.r())
    }

    // https://dom.spec.whatwg.org/#dom-domimplementation-hasfeature
    fn HasFeature(self, _feature: DOMString, _version: DOMString) -> bool {
        true
    }
}
