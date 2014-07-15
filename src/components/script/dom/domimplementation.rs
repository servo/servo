/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMImplementationBinding;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::error::{Fallible, InvalidCharacter, NamespaceError};
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Root, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::document::{Document, HTMLDocument, NonHTMLDocument, DocumentMethods};
use dom::documenttype::DocumentType;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::{Node, NodeMethods};
use dom::text::Text;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct DOMImplementation {
    document: JS<Document>,
    reflector_: Reflector,
}

impl DOMImplementation {
    pub fn new_inherited(document: &JSRef<Document>) -> DOMImplementation {
        DOMImplementation {
            document: JS::from_rooted(document),
            reflector_: Reflector::new(),
        }
    }

    pub fn new(document: &JSRef<Document>) -> Temporary<DOMImplementation> {
        let window = document.window.root();
        reflect_dom_object(box DOMImplementation::new_inherited(document),
                           &Window(*window),
                           DOMImplementationBinding::Wrap)
    }
}

impl Reflectable for DOMImplementation {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

pub trait DOMImplementationMethods {
    fn CreateDocumentType(&self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<Temporary<DocumentType>>;
    fn CreateDocument(&self, namespace: Option<DOMString>, qname: DOMString,
                      mut maybe_doctype: Option<JSRef<DocumentType>>) -> Fallible<Temporary<Document>>;
    fn CreateHTMLDocument(&self, title: Option<DOMString>) -> Temporary<Document>;
}

// http://dom.spec.whatwg.org/#domimplementation
impl<'a> DOMImplementationMethods for JSRef<'a, DOMImplementation> {
    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    fn CreateDocumentType(&self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<Temporary<DocumentType>> {
        match xml_name_type(qname.as_slice()) {
            // Step 1.
            InvalidXMLName => Err(InvalidCharacter),
            // Step 2.
            Name => Err(NamespaceError),
            // Step 3.
            QName => {
                let document = self.document.root();
                Ok(DocumentType::new(qname, Some(pubid), Some(sysid), &*document))
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    fn CreateDocument(&self, namespace: Option<DOMString>, qname: DOMString,
                      maybe_doctype: Option<JSRef<DocumentType>>) -> Fallible<Temporary<Document>> {
        let doc = self.document.root();
        let win = doc.window.root();

        // Step 1.
        let doc = Document::new(&win.root_ref(), None, NonHTMLDocument, None).root();
        // Step 2-3.
        let maybe_elem = if qname.is_empty() {
            None
        } else {
            match doc.CreateElementNS(namespace, qname) {
                Err(error) => return Err(error),
                Ok(elem) => Some(elem)
            }
        };

        {
            let doc_node: &JSRef<Node> = NodeCast::from_ref(&*doc);

            // Step 4.
            match maybe_doctype {
                None => (),
                Some(ref doctype) => {
                    let doc_type: &JSRef<Node> = NodeCast::from_ref(doctype);
                    assert!(doc_node.AppendChild(doc_type).is_ok())
                }
            }

            // Step 5.
            match maybe_elem.root() {
                None => (),
                Some(elem) => {
                    assert!(doc_node.AppendChild(NodeCast::from_ref(&*elem)).is_ok())
                }
            }
        }

        // Step 6.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 7.
        Ok(Temporary::from_rooted(&*doc))
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    fn CreateHTMLDocument(&self, title: Option<DOMString>) -> Temporary<Document> {
        let document = self.document.root();
        let win = document.window.root();

        // Step 1-2.
        let doc = Document::new(&win.root_ref(), None, HTMLDocument, None).root();
        let doc_node: &JSRef<Node> = NodeCast::from_ref(&*doc);

        {
            // Step 3.
            let doc_type = DocumentType::new("html".to_string(), None, None, &*doc).root();
            assert!(doc_node.AppendChild(NodeCast::from_ref(&*doc_type)).is_ok());
        }

        {
            // Step 4.
            let doc_html: Root<Node> = NodeCast::from_temporary(HTMLHtmlElement::new("html".to_string(), &*doc)).root();
            let doc_html = doc_html.deref();
            assert!(doc_node.AppendChild(doc_html).is_ok());

            {
                // Step 5.
                let doc_head: Root<Node> = NodeCast::from_temporary(HTMLHeadElement::new("head".to_string(), &*doc)).root();
                let doc_head = doc_head.deref();
                assert!(doc_html.AppendChild(doc_head).is_ok());

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let doc_title: Root<Node> = NodeCast::from_temporary(HTMLTitleElement::new("title".to_string(), &*doc)).root();
                        let doc_title = doc_title.deref();
                        assert!(doc_head.AppendChild(doc_title).is_ok());

                        // Step 6.2.
                        let title_text: Root<Text> = Text::new(title_str, &*doc).root();
                        let title_text = title_text.deref();
                        assert!(doc_title.AppendChild(NodeCast::from_ref(title_text)).is_ok());
                    }
                }
            }

            // Step 7.
            let doc_body: Root<HTMLBodyElement> = HTMLBodyElement::new("body".to_string(), &*doc).root();
            let doc_body = doc_body.deref();
            assert!(doc_html.AppendChild(NodeCast::from_ref(doc_body)).is_ok());
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        Temporary::from_rooted(&*doc)
    }
}
