/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{DocumentDerived, EventCast, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::{DocumentBase, NodeCast, DocumentCast};
use dom::bindings::codegen::InheritTypes::{HTMLHeadElementCast, TextCast, ElementCast};
use dom::bindings::codegen::DocumentBinding;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{Reflectable, Reflector, Traceable, reflect_dom_object2};
use dom::bindings::utils::{ErrorResult, Fallible, NotSupported, InvalidCharacter, HierarchyRequest};
use dom::bindings::utils::DOMString;
use dom::bindings::utils::{xml_name_type, InvalidXMLName};
use dom::comment::Comment;
use dom::documentfragment::DocumentFragment;
use dom::element::{Element};
use dom::element::{HTMLHtmlElementTypeId, HTMLHeadElementTypeId, HTMLTitleElementTypeId};
use dom::element::{HTMLBodyElementTypeId, HTMLFrameSetElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::htmlheadelement::HTMLHeadElement;
use dom::htmltitleelement::HTMLTitleElement;
use dom::mouseevent::MouseEvent;
use dom::namespace::Null;
use dom::node::{Node, ElementNodeTypeId, DocumentNodeTypeId, NodeHelpers, INode};
use dom::text::Text;
use dom::uievent::UIEvent;
use dom::window::Window;
use html::hubbub_html_parser::build_element_from_tag;
use layout_interface::{DocumentDamageLevel, ContentChangedDocumentDamage};

use js::jsapi::{JSObject, JSContext, JSTracer};
use std::ascii::StrAsciiExt;
use std::hashmap::HashMap;
use std::str::eq_slice;

#[deriving(Eq)]
pub enum DocumentTypeId {
    PlainDocumentTypeId,
    HTMLDocumentTypeId
}

#[deriving(Eq)]
pub enum DocumentType {
    HTML,
    SVG,
    XML
}

pub struct Document {
    node: Node,
    reflector_: Reflector,
    window: JSManaged<Window>,
    doctype: DocumentType,
    title: ~str,
    idmap: HashMap<DOMString, JSManaged<Element>>
}

impl DocumentDerived for EventTarget {
    fn is_document(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(DocumentNodeTypeId(_)) => true,
            _ => false
        }
    }
}

impl Document {
    pub fn reflect_document<D: Reflectable+DocumentBase>
            (document:  ~D,
             window:    JSManaged<Window>,
             wrap_fn:   extern "Rust" fn(*JSContext, *JSObject, ~D) -> *JSObject)
             -> JSManaged<D> {
        assert!(document.reflector().get_jsobject().is_null());
        let raw_doc = reflect_dom_object2(document, window.value(), wrap_fn);
        assert!(raw_doc.reflector().get_jsobject().is_not_null());

        let document = DocumentCast::from(raw_doc);
        let mut node: JSManaged<Node> = NodeCast::from(document);
        node.mut_value().set_owner_doc(document);
        raw_doc
    }

    pub fn new_inherited(window: JSManaged<Window>, doctype: DocumentType) -> Document {
        let node_type = match doctype {
            HTML => HTMLDocumentTypeId,
            SVG | XML => PlainDocumentTypeId
        };
        Document {
            node: Node::new_without_doc(DocumentNodeTypeId(node_type)),
            reflector_: Reflector::new(),
            window: window,
            doctype: doctype,
            title: ~"",
            idmap: HashMap::new()
        }
    }

    pub fn new(window: JSManaged<Window>, doctype: DocumentType) -> JSManaged<Document> {
        let document = Document::new_inherited(window, doctype);
        Document::reflect_document(~document, window, DocumentBinding::Wrap)
    }
}

impl Document {
    pub fn Constructor(owner: JSManaged<Window>) -> Fallible<JSManaged<Document>> {
        Ok(Document::new(owner, XML))
    }
}

impl Reflectable for Document {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
    }
}

impl Document {
    pub fn GetDocumentElement(&self) -> Option<JSManaged<Element>> {
        self.node.child_elements().next()
    }

    pub fn GetElementsByTagName(&self, tag: DOMString) -> JSManaged<HTMLCollection> {
        self.createHTMLCollection(|elem| eq_slice(elem.tag_name, tag))
    }

    pub fn GetElementsByTagNameNS(&self, _ns: Option<DOMString>, _tag: DOMString) -> JSManaged<HTMLCollection> {
        HTMLCollection::new(self.window, ~[])
    }

    pub fn GetElementsByClassName(&self, _class: DOMString) -> JSManaged<HTMLCollection> {
        HTMLCollection::new(self.window, ~[])
    }

    pub fn GetElementById(&self, id: DOMString) -> Option<JSManaged<Element>> {
        // TODO: "in tree order, within the context object's tree"
        // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
        match self.idmap.find_equiv(&id) {
            None => None,
            Some(node) => Some(*node),
        }
    }

    pub fn CreateElement(&self, abstract_self: JSManaged<Document>, local_name: DOMString)
                         -> Fallible<JSManaged<Element>> {
        if xml_name_type(local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }
        let local_name = local_name.to_ascii_lower();
        Ok(build_element_from_tag(local_name, abstract_self))
    }

    pub fn CreateDocumentFragment(&self, abstract_self: JSManaged<Document>) -> JSManaged<DocumentFragment> {
        DocumentFragment::new(abstract_self)
    }

    pub fn CreateTextNode(&self, abstract_self: JSManaged<Document>, data: DOMString)
                          -> JSManaged<Text> {
        Text::new(data, abstract_self)
    }

    pub fn CreateComment(&self, abstract_self: JSManaged<Document>, data: DOMString) -> JSManaged<Comment> {
        Comment::new(data, abstract_self)
    }

    pub fn CreateEvent(&self, interface: DOMString) -> Fallible<JSManaged<Event>> {
        match interface.as_slice() {
            "UIEvents" => Ok(EventCast::from(UIEvent::new(self.window))),
            "MouseEvents" => Ok(EventCast::from(MouseEvent::new(self.window))),
            "HTMLEvents" => Ok(Event::new(self.window)),
            _ => Err(NotSupported)
        }
    }

    pub fn Title(&self, _: JSManaged<Document>) -> DOMString {
        let mut title = ~"";
        match self.doctype {
            SVG => {
                fail!("no SVG document yet")
            },
            _ => {
                match self.GetDocumentElement() {
                    None => {},
                    Some(root) => {
                        let root: JSManaged<Node> = NodeCast::from(root);
                        let title_type = ElementNodeTypeId(HTMLTitleElementTypeId);
                        let title_elem = root.traverse_preorder().find(|node| node.type_id() == title_type);
                        for node in title_elem.iter() {
                            for child in node.children() {
                                let text: JSManaged<Text> = TextCast::to(child);
                                title = title + text.value().element.Data();
                            }
                        }
                    }
                }
            }
        }
        let v: ~[&str] = title.words().collect();
        title = v.connect(" ");
        title = title.trim().to_owned();
        title
    }

    pub fn SetTitle(&self, abstract_self: JSManaged<Document>, title: DOMString) -> ErrorResult {
        match self.doctype {
            SVG => {
                fail!("no SVG document yet")
            },
            _ => {
                match self.GetDocumentElement() {
                    None => {},
                    Some(root) => {
                        let root: JSManaged<Node> = NodeCast::from(root);
                        let head_type = ElementNodeTypeId(HTMLHeadElementTypeId);
                        let mut children = root.traverse_preorder();
                        let mut head = children.find(|child| child.value().type_id == head_type);
                        for node in head.mut_iter() {
                            let mut has_title = false;
                            let title_type = ElementNodeTypeId(HTMLTitleElementTypeId);
                            let mut children = node.value().children();
                            let mut title_node = children.find(|child| child.value().type_id == title_type);
                            for child in title_node.mut_iter() {
                                has_title = true;
                                for title_child in child.value().children() {
                                    child.RemoveChild(title_child);
                                }
                                let new_text = self.CreateTextNode(abstract_self, title.clone());
                                child.AppendChild(NodeCast::from(new_text));
                            }
                            if !has_title {
                                let new_title: JSManaged<Node> =
                                    NodeCast::from(HTMLTitleElement::new(~"title", abstract_self));
                                let new_text = self.CreateTextNode(abstract_self, title.clone());
                                new_title.AppendChild(NodeCast::from(new_text));
                                node.AppendChild(new_title);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn get_html_element(&self) -> Option<JSManaged<HTMLElement>> {
        self.GetDocumentElement().filtered(|root| {
            root.value().node.type_id == ElementNodeTypeId(HTMLHtmlElementTypeId)
        }).map(|elem| HTMLElementCast::to(elem))
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-head
    pub fn GetHead(&self) -> Option<JSManaged<HTMLHeadElement>> {
        self.get_html_element().and_then(|root| {
            root.value().element.node.children().find(|child| {
                child.value().type_id == ElementNodeTypeId(HTMLHeadElementTypeId)
            }).map(|node| {
                let head: JSManaged<HTMLHeadElement> = HTMLHeadElementCast::to(node);
                head
            })
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn GetBody(&self, _: JSManaged<Document>) -> Option<JSManaged<HTMLElement>> {
        match self.get_html_element() {
            None => None,
            Some(root) => {
                root.value().element.node.children().find(|child| {
                    match child.value().type_id {
                        ElementNodeTypeId(HTMLBodyElementTypeId) |
                        ElementNodeTypeId(HTMLFrameSetElementTypeId) => true,
                        _ => false
                    }
                }).map(|node| HTMLElementCast::to(node))
            }
        }
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn SetBody(&self, abstract_self: JSManaged<Document>, new_body: Option<JSManaged<HTMLElement>>) -> ErrorResult {
        // Step 1.
        match new_body {
            Some(node) => {
                match node.value().element.node.type_id {
                    ElementNodeTypeId(HTMLBodyElementTypeId) | ElementNodeTypeId(HTMLFrameSetElementTypeId) => {}
                    _ => return Err(HierarchyRequest)
                }
            }
            None => return Err(HierarchyRequest)
        }

        // Step 2.
        let old_body: Option<JSManaged<HTMLElement>> = self.GetBody(abstract_self);
        if old_body == new_body {
            return Ok(());
        }

        // Step 3.
        match self.get_html_element() {
            // Step 4.
            None => return Err(HierarchyRequest),
            Some(root) => {
                let new_body: JSManaged<Node> = NodeCast::from(new_body.unwrap());
                let root: JSManaged<Node> = NodeCast::from(root);
                match old_body {
                    Some(child) => {
                        let child: JSManaged<Node> = NodeCast::from(child);
                        root.ReplaceChild(new_body, child)
                    }
                    None => root.AppendChild(new_body)
                };
            }
        }
        Ok(())
    }

    pub fn GetElementsByName(&self, name: DOMString) -> JSManaged<HTMLCollection> {
        self.createHTMLCollection(|elem|
            elem.get_attr(Null, "name").is_some() && eq_slice(elem.get_attr(Null, "name").unwrap(), name))
    }

    pub fn createHTMLCollection(&self, callback: |elem: &Element| -> bool) -> JSManaged<HTMLCollection> {
        let mut elements = ~[];
        match self.GetDocumentElement() {
            None => {},
            Some(root) => {
                let root: JSManaged<Node> = NodeCast::from(root);
                for child in root.traverse_preorder() {
                    if child.is_element() {
                        let elem: JSManaged<Element> = ElementCast::to(child);
                        if callback(elem.value()) {
                            elements.push(elem);
                        }
                    }
                }
            }
        }
        HTMLCollection::new(self.window, elements)
    }

    pub fn content_changed(&self) {
        self.damage_and_reflow(ContentChangedDocumentDamage);
    }

    pub fn damage_and_reflow(&self, damage: DocumentDamageLevel) {
        self.window.value().damage_and_reflow(damage);
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        self.window.value().wait_until_safe_to_modify_dom();
    }

    pub fn register_nodes_with_id(&mut self, root: &JSManaged<Element>) {
        foreach_ided_elements(root, |id: &DOMString, abstract_node: &JSManaged<Element>| {
            // TODO: "in tree order, within the context object's tree"
            // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
            self.idmap.find_or_insert(id.clone(), *abstract_node);
        });
    }

    pub fn unregister_nodes_with_id(&mut self, root: &JSManaged<Element>) {
        foreach_ided_elements(root, |id: &DOMString, _| {
            // TODO: "in tree order, within the context object's tree"
            // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
            self.idmap.pop(id);
        });
    }

    pub fn update_idmap(&mut self,
                        abstract_self: JSManaged<Element>,
                        new_id: Option<DOMString>,
                        old_id: Option<DOMString>) {
        // remove old ids:
        // * if the old ones are not same as the new one,
        // * OR if the new one is none.
        match old_id {
            Some(ref old_id) if new_id.is_none() ||
                                (*new_id.get_ref() != *old_id) => {
                self.idmap.remove(old_id);
            }
            _ => ()
        }

        match new_id {
            Some(new_id) => {
                // TODO: support the case if multiple elements
                // which haves same id are in the same document.
                self.idmap.mangle(new_id, abstract_self,
                                  |_, new_node: JSManaged<Element>| -> JSManaged<Element> {
                                      new_node
                                  },
                                  |_, old_node: &mut JSManaged<Element>, new_node: JSManaged<Element>| {
                                      *old_node = new_node;
                                  });
            }
            None => ()
        }
    }
}

#[inline(always)]
fn foreach_ided_elements(root: &JSManaged<Element>, callback: |&DOMString, &JSManaged<Element>|) {
    let root: JSManaged<Node> = NodeCast::from(*root);
    for node in root.traverse_preorder() {
        if !node.is_element() {
            continue;
        }
        let element: JSManaged<Element> = ElementCast::to(node);
        match element.value().get_attr(Null, "id") {
            Some(id) => {
                callback(&id.to_str(), &element);
            }
            None => ()
        }
    }
}

impl Traceable for Document {
    fn trace(&self, tracer: *mut JSTracer) {
        self.node.trace(tracer);
    }
}
