/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMImplementationBinding;
use dom::bindings::utils::{DOMString, Reflector, Reflectable, reflect_dom_object};
use dom::bindings::utils::{Fallible, InvalidCharacter, NamespaceError};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::document::{AbstractDocument, HTML, HTMLDocumentTypeId};
use dom::documenttype::DocumentType;
use dom::htmldocument::HTMLDocument;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::{AbstractNode, DocumentNodeTypeId};
use dom::text::Text;
use dom::window::Window;

pub struct DOMImplementation {
    owner: @mut Window,
    reflector_: Reflector
}

impl DOMImplementation {
    pub fn new_inherited(owner: @mut Window) -> DOMImplementation {
        DOMImplementation {
            owner: owner,
            reflector_: Reflector::new()
        }
    }

    pub fn new(owner: @mut Window) -> @mut DOMImplementation {
        reflect_dom_object(@mut DOMImplementation::new_inherited(owner), owner,
                           DOMImplementationBinding::Wrap)
    }
}

impl Reflectable for DOMImplementation {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

// http://dom.spec.whatwg.org/#domimplementation
impl DOMImplementation {
    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
    pub fn CreateDocumentType(&self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<AbstractNode> {
        match xml_name_type(qname) {
            // Step 1.
            InvalidXMLName => Err(InvalidCharacter),
            // Step 2.
            Name => Err(NamespaceError),
            // Step 3.
            QName => Ok(DocumentType::new(qname, Some(pubid), Some(sysid), self.owner.Document()))
        }
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    pub fn CreateHTMLDocument(&self, title: Option<DOMString>) -> AbstractDocument {
        // Step 1-2.
        let abstract_doc = HTMLDocument::new(self.owner);
        assert!(abstract_doc.document().doctype == HTML);

        let abstract_node = AbstractNode::from_document(abstract_doc);
        assert!(abstract_node.type_id() == DocumentNodeTypeId(HTMLDocumentTypeId));

        {
            // Step 3.
            let doc_type = DocumentType::new(~"html", None, None, abstract_doc);
            abstract_node.AppendChild(doc_type);
        }

        {
            // Step 4.
            let doc_html = HTMLHtmlElement::new(~"html", abstract_doc);
            abstract_node.AppendChild(doc_html);

            {
                // Step 5.
                let doc_head = HTMLHeadElement::new(~"head", abstract_doc);
                doc_html.AppendChild(doc_head);

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let doc_title = HTMLTitleElement::new(~"title", abstract_doc);
                        doc_head.AppendChild(doc_title);

                        // Step 6.2.
                        let title_text = Text::new(title_str, abstract_doc);
                        doc_title.AppendChild(title_text);
                    }
                }
            }

            // Step 7.
            let doc_body = HTMLBodyElement::new(~"body", abstract_doc);
            doc_html.AppendChild(doc_body);
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        abstract_doc
    }
}
