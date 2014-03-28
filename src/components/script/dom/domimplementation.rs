/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::DOMImplementationBinding;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::{JS, JSRef, RootCollection};
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

    pub fn new(owner: &JSRef<Window>) -> JS<DOMImplementation> {
        reflect_dom_object(~DOMImplementation::new_inherited(owner.unrooted()), owner,
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
        let roots = RootCollection::new();
        match xml_name_type(qname) {
            // Step 1.
            InvalidXMLName => Err(InvalidCharacter),
            // Step 2.
            Name => Err(NamespaceError),
            // Step 3.
            QName => {
                let document = self.owner.get().Document();
                let document = document.root(&roots);
                Ok(DocumentType::new(qname, Some(pubid), Some(sysid), &document.root_ref()))
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createdocument
    pub fn CreateDocument(&self, namespace: Option<DOMString>, qname: DOMString,
                          mut maybe_doctype: Option<JSRef<DocumentType>>) -> Fallible<JS<Document>> {
        let roots = RootCollection::new();
        let win = self.owner.root(&roots);

        // Step 1.
        let doc = Document::new(&win.root_ref(), None, NonHTMLDocument, None);
        let doc_root = doc.root(&roots);
        let mut doc_node: JS<Node> = NodeCast::from(&doc);

        // Step 2-3.
        let mut maybe_elem = if qname.is_empty() {
            None
        } else {
            match doc.get().CreateElementNS(&doc_root.root_ref(), namespace, qname) {
                Err(error) => return Err(error),
                Ok(elem) => Some(elem)
            }
        };

        // Step 4.
        match maybe_doctype {
            None => (),
            Some(ref mut doctype) => assert!(doc_node.AppendChild(NodeCast::from_mut_ref(doctype)).is_ok())
        }

        // Step 5.
        match maybe_elem {
            None => (),
            Some(ref elem) => {
                let elem = elem.root(&roots);
                assert!(doc_node.AppendChild(NodeCast::from_mut_ref(&mut elem.root_ref())).is_ok())
            }
        }

        // Step 6.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 7.
        Ok(doc)
    }

    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    pub fn CreateHTMLDocument(&self, title: Option<DOMString>) -> JS<Document> {
        let roots = RootCollection::new();
        let owner = self.owner.root(&roots);

        // Step 1-2.
        let doc = Document::new(&owner.root_ref(), None, HTMLDocument, None);
        let doc_root = doc.root(&roots);
        let mut doc_node: JS<Node> = NodeCast::from(&doc);

        {
            // Step 3.
            let doc_type = DocumentType::new(~"html", None, None, &doc_root.root_ref());
            let doc_type = doc_type.root(&roots);
            assert!(doc_node.AppendChild(NodeCast::from_mut_ref(&mut doc_type.root_ref())).is_ok());
        }

        {
            // Step 4.
            let mut doc_html = NodeCast::from(&HTMLHtmlElement::new(~"html", &doc_root.root_ref()));
            let doc_html_root = {doc_html.root(&roots)};
            assert!(doc_node.AppendChild(&mut doc_html_root.root_ref()).is_ok());

            {
                // Step 5.
                let mut doc_head = NodeCast::from(&HTMLHeadElement::new(~"head", &doc_root.root_ref()));
                let doc_head_root = doc_head.root(&roots);
                assert!(doc_html.AppendChild(&mut doc_head_root.root_ref()).is_ok());

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let mut doc_title = NodeCast::from(&HTMLTitleElement::new(~"title", &doc_root.root_ref()));
                        let doc_title_root = doc_title.root(&roots);
                        assert!(doc_head.AppendChild(&mut doc_title_root.root_ref()).is_ok());

                        // Step 6.2.
                        let title_text = Text::new(title_str, &doc_root.root_ref());
                        let title_text = title_text.root(&roots);
                        assert!(doc_title.AppendChild(NodeCast::from_mut_ref(&mut title_text.root_ref())).is_ok());
                    }
                }
            }

            // Step 7.
            let doc_body = HTMLBodyElement::new(~"body", &doc_root.root_ref());
            let doc_body = doc_body.root(&roots);
            assert!(doc_html.AppendChild(NodeCast::from_mut_ref(&mut doc_body.root_ref())).is_ok());
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        doc
    }
}
