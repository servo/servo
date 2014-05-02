/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::DOMImplementationBinding;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::bindings::error::{Fallible, InvalidCharacter, NamespaceError};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::document::{Document, HTMLDocument, NonHTMLDocument};
use dom::documenttype::DocumentType;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::node::{Node, INode};
use dom::text::Text;
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct DOMImplementation {
    pub owner: JS<Window>,
    pub reflector_: Reflector,
}

impl DOMImplementation {
    pub fn new_inherited(owner: JS<Window>) -> DOMImplementation {
        DOMImplementation {
            owner: owner,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(owner: &JS<Window>) -> JS<DOMImplementation> {
        reflect_dom_object(~DOMImplementation::new_inherited(owner.clone()), owner,
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
    pub fn CreateDocumentType(&self, qname: DOMString, pubid: DOMString, sysid: DOMString) -> Fallible<JS<DocumentType>> {
        match xml_name_type(qname) {
            // Step 1.
            InvalidXMLName => Err(InvalidCharacter),
            // Step 2.
            Name => Err(NamespaceError),
            // Step 3.
            QName => Ok(DocumentType::new(qname, Some(pubid), Some(sysid), &self.owner.get().Document()))
        }
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    pub fn CreateDocument(&self, namespace: Option<DOMString>, qname: DOMString,
                          maybe_doctype: Option<JS<DocumentType>>) -> Fallible<JS<Document>> {
        // Step 1.
        let doc = Document::new(&self.owner, None, NonHTMLDocument, None);
        let mut doc_node: JS<Node> = NodeCast::from(&doc);

        // Step 2-3.
        let maybe_elem = if qname.is_empty() {
            None
        } else {
            match doc.get().CreateElementNS(&doc, namespace, qname) {
                Err(error) => return Err(error),
                Ok(elem) => Some(elem)
            }
        };

        // Step 4.
        match maybe_doctype {
            None => (),
            Some(ref doctype) => assert!(doc_node.AppendChild(&mut NodeCast::from(doctype)).is_ok())
        }

        // Step 5.
        match maybe_elem {
            None => (),
            Some(ref elem) => assert!(doc_node.AppendChild(&mut NodeCast::from(elem)).is_ok())
        }

        // Step 6.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 7.
        Ok(doc)
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    pub fn CreateHTMLDocument(&self, title: Option<DOMString>) -> JS<Document> {
        // Step 1-2.
        let doc = Document::new(&self.owner, None, HTMLDocument, None);
        let mut doc_node: JS<Node> = NodeCast::from(&doc);

        {
            // Step 3.
            let doc_type = DocumentType::new(~"html", None, None, &doc);
            assert!(doc_node.AppendChild(&mut NodeCast::from(&doc_type)).is_ok());
        }

        {
            // Step 4.
            let mut doc_html = NodeCast::from(&HTMLHtmlElement::new(~"html", &doc));
            assert!(doc_node.AppendChild(&mut doc_html).is_ok());

            {
                // Step 5.
                let mut doc_head = NodeCast::from(&HTMLHeadElement::new(~"head", &doc));
                assert!(doc_html.AppendChild(&mut doc_head).is_ok());

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let mut doc_title = NodeCast::from(&HTMLTitleElement::new(~"title", &doc));
                        assert!(doc_head.AppendChild(&mut doc_title).is_ok());

                        // Step 6.2.
                        let title_text = Text::new(title_str, &doc);
                        assert!(doc_title.AppendChild(&mut NodeCast::from(&title_text)).is_ok());
                    }
                }
            }

            // Step 7.
            let doc_body = HTMLBodyElement::new(~"body", &doc);
            assert!(doc_html.AppendChild(&mut NodeCast::from(&doc_body)).is_ok());
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        doc
    }
}
