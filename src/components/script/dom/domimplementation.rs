/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMImplementationBinding;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflector, Reflectable, reflect_dom_object};
use dom::bindings::error::{Fallible, InvalidCharacter, NamespaceError};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::document::{Document, HTMLDocument};
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
    owner: JS<Window>,
    reflector_: Reflector,
}

impl DOMImplementation {
    pub fn new_inherited(owner: JS<Window>) -> DOMImplementation {
        DOMImplementation {
            owner: owner,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(owner: &JS<Window>) -> JS<DOMImplementation> {
        reflect_dom_object(~DOMImplementation::new_inherited(owner.clone()), owner.get(),
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

    // http://dom.spec.whatwg.org/#dom-domimplementation-createhtmldocument
    pub fn CreateHTMLDocument(&self, title: Option<DOMString>) -> JS<Document> {
        // Step 1-2.
        let doc = Document::new(&self.owner, None, HTMLDocument, None);
        let mut doc_node: JS<Node> = NodeCast::from(&doc);

        {
            // Step 3.
            let doc_type = DocumentType::new(~"html", None, None, &doc);
            doc_node.AppendChild(&mut NodeCast::from(&doc_type));
        }

        {
            // Step 4.
            let mut doc_html = NodeCast::from(&HTMLHtmlElement::new(~"html", &doc));
            doc_node.AppendChild(&mut doc_html);

            {
                // Step 5.
                let mut doc_head = NodeCast::from(&HTMLHeadElement::new(~"head", &doc));
                doc_html.AppendChild(&mut doc_head);

                // Step 6.
                match title {
                    None => (),
                    Some(title_str) => {
                        // Step 6.1.
                        let mut doc_title = NodeCast::from(&HTMLTitleElement::new(~"title", &doc));
                        doc_head.AppendChild(&mut doc_title);

                        // Step 6.2.
                        let title_text = Text::new(title_str, &doc);
                        doc_title.AppendChild(&mut NodeCast::from(&title_text));
                    }
                }
            }

            // Step 7.
            let doc_body = HTMLBodyElement::new(~"body", &doc);
            doc_html.AppendChild(&mut NodeCast::from(&doc_body));
        }

        // Step 8.
        // FIXME: https://github.com/mozilla/servo/issues/1522

        // Step 9.
        doc
    }
}
