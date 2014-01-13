/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::comment::Comment;
use dom::bindings::codegen::DocumentBinding;
use dom::bindings::utils::{Reflectable, Reflector, Traceable, reflect_dom_object};
use dom::bindings::utils::{ErrorResult, Fallible, NotSupported, InvalidCharacter, HierarchyRequest};
use dom::bindings::utils::DOMString;
use dom::bindings::utils::{xml_name_type, InvalidXMLName};
use dom::documentfragment::DocumentFragment;
use dom::element::{Element};
use dom::element::{HTMLHtmlElementTypeId, HTMLHeadElementTypeId, HTMLTitleElementTypeId, HTMLBodyElementTypeId, HTMLFrameSetElementTypeId};
use dom::event::{AbstractEvent, Event};
use dom::htmlcollection::HTMLCollection;
use dom::htmldocument::HTMLDocument;
use dom::mouseevent::MouseEvent;
use dom::namespace::Null;
use dom::node::{AbstractNode, Node, ElementNodeTypeId, DocumentNodeTypeId};
use dom::text::Text;
use dom::uievent::UIEvent;
use dom::window::Window;
use dom::htmltitleelement::HTMLTitleElement;
use html::hubbub_html_parser::build_element_from_tag;
use layout_interface::{DocumentDamageLevel, ContentChangedDocumentDamage};

use js::jsapi::{JSObject, JSContext, JSTracer};
use std::ascii::StrAsciiExt;
use std::cast;
use std::hashmap::HashMap;
use std::str::eq_slice;
use std::unstable::raw::Box;

#[deriving(Eq)]
pub enum DocumentTypeId {
    PlainDocumentTypeId,
    HTMLDocumentTypeId
}

#[deriving(Eq)]
pub struct AbstractDocument {
    document: *mut Box<Document>
}

impl AbstractDocument {
    pub fn document<'a>(&'a self) -> &'a Document {
        unsafe {
            &(*self.document).data
        }
    }

    pub fn mut_document<'a>(&'a self) -> &'a mut Document {
        unsafe {
            &mut (*self.document).data
        }
    }

    unsafe fn transmute<T, R>(&self, f: |&T| -> R) -> R {
        let box_: *Box<T> = cast::transmute(self.document);
        f(&(*box_).data)
    }

    pub fn with_html<R>(&self, callback: |&HTMLDocument| -> R) -> R {
        match self.document().doctype {
            HTML => unsafe { self.transmute(callback) },
            _ => fail!("attempt to downcast a non-HTMLDocument to HTMLDocument")
        }
    }

    pub fn from_box<T>(ptr: *mut Box<T>) -> AbstractDocument {
        AbstractDocument {
            document: ptr as *mut Box<Document>
        }
    }
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
    window: @mut Window,
    doctype: DocumentType,
    title: ~str,
    idmap: HashMap<DOMString, AbstractNode>
}

impl Document {
    pub fn reflect_document<D: Reflectable>
            (document:  @mut D,
             window:    @mut Window,
             wrap_fn:   extern "Rust" fn(*JSContext, *JSObject, @mut D) -> *JSObject)
             -> AbstractDocument {
        assert!(document.reflector().get_jsobject().is_null());
        let document = reflect_dom_object(document, window, wrap_fn);
        assert!(document.reflector().get_jsobject().is_not_null());

        // JS object now owns the Document, so transmute_copy is needed
        let abstract = AbstractDocument {
            document: unsafe { cast::transmute_copy(&document) }
        };
        abstract.mut_document().node.set_owner_doc(abstract);
        abstract
    }

    pub fn new_inherited(window: @mut Window, doctype: DocumentType) -> Document {
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

    pub fn new(window: @mut Window, doctype: DocumentType) -> AbstractDocument {
        let document = Document::new_inherited(window, doctype);
        Document::reflect_document(@mut document, window, DocumentBinding::Wrap)
    }
}

impl Document {
    pub fn Constructor(owner: @mut Window) -> Fallible<AbstractDocument> {
        Ok(Document::new(owner, XML))
    }
}

impl Reflectable for AbstractDocument {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.document().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.mut_document().mut_reflector()
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
    pub fn GetDocumentElement(&self) -> Option<AbstractNode> {
        self.node.child_elements().next()
    }

    pub fn GetElementsByTagName(&self, tag: DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem| eq_slice(elem.tag_name, tag))
    }

    pub fn GetElementsByTagNameNS(&self, _ns: Option<DOMString>, _tag: DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.window, ~[])
    }

    pub fn GetElementsByClassName(&self, _class: DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.window, ~[])
    }

    pub fn GetElementById(&self, id: DOMString) -> Option<AbstractNode> {
        // TODO: "in tree order, within the context object's tree"
        // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
        match self.idmap.find_equiv(&id) {
            None => None,
            Some(node) => Some(*node),
        }
    }

    pub fn CreateElement(&self, abstract_self: AbstractDocument, local_name: DOMString)
                         -> Fallible<AbstractNode> {
        if xml_name_type(local_name) == InvalidXMLName {
            debug!("Not a valid element name");
            return Err(InvalidCharacter);
        }
        let local_name = local_name.to_ascii_lower();
        Ok(build_element_from_tag(local_name, abstract_self))
    }

    pub fn CreateDocumentFragment(&self, abstract_self: AbstractDocument) -> AbstractNode {
        DocumentFragment::new(abstract_self)
    }

    pub fn CreateTextNode(&self, abstract_self: AbstractDocument, data: DOMString)
                          -> AbstractNode {
        Text::new(data, abstract_self)
    }

    pub fn CreateComment(&self, abstract_self: AbstractDocument, data: DOMString) -> AbstractNode {
        Comment::new(data, abstract_self)
    }

    pub fn CreateEvent(&self, interface: DOMString) -> Fallible<AbstractEvent> {
        match interface.as_slice() {
            "UIEvents" => Ok(UIEvent::new(self.window)),
            "MouseEvents" => Ok(MouseEvent::new(self.window)),
            "HTMLEvents" => Ok(Event::new(self.window)),
            _ => Err(NotSupported)
        }
    }

    pub fn Title(&self, _: AbstractDocument) -> DOMString {
        let mut title = ~"";
        match self.doctype {
            SVG => {
                fail!("no SVG document yet")
            },
            _ => {
                match self.GetDocumentElement() {
                    None => {},
                    Some(root) => {
                        for node in root.traverse_preorder() {
                            if node.type_id() != ElementNodeTypeId(HTMLTitleElementTypeId) {
                                continue;
                            }
                            for child in node.children() {
                                if child.is_text() {
                                    child.with_imm_text(|text| {
                                        title = title + text.element.Data();
                                    });
                                }
                            }
                            break;
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

    pub fn SetTitle(&self, abstract_self: AbstractDocument, title: DOMString) -> ErrorResult {
        match self.doctype {
            SVG => {
                fail!("no SVG document yet")
            },
            _ => {
                match self.GetDocumentElement() {
                    None => {},
                    Some(root) => {
                        for node in root.traverse_preorder() {
                            if node.type_id() != ElementNodeTypeId(HTMLHeadElementTypeId) {
                                continue;
                            }
                            let mut has_title = false;
                            for child in node.children() {
                                if child.type_id() != ElementNodeTypeId(HTMLTitleElementTypeId) {
                                    continue;
                                }
                                has_title = true;
                                for title_child in child.children() {
                                    child.RemoveChild(title_child);
                                }
                                child.AppendChild(self.CreateTextNode(abstract_self, title.clone()));
                                break;
                            }
                            if !has_title {
                                let new_title = HTMLTitleElement::new(~"title", abstract_self);
                                new_title.AppendChild(self.CreateTextNode(abstract_self, title.clone()));
                                node.AppendChild(new_title);
                            }
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn get_html_element(&self) -> Option<AbstractNode> {
        self.GetDocumentElement().filtered(|root| {
            match root.type_id() {
                ElementNodeTypeId(HTMLHtmlElementTypeId) => true,
                _ => false
            }
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-head
    pub fn GetHead(&self) -> Option<AbstractNode> {
        self.get_html_element().and_then(|root| {
            root.children().find(|child| {
                child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId)
            })
        })
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn GetBody(&self, _: AbstractDocument) -> Option<AbstractNode> {
        match self.get_html_element() {
            None => None,
            Some(root) => {
                root.children().find(|child| {
                    match child.type_id() {
                        ElementNodeTypeId(HTMLBodyElementTypeId) |
                        ElementNodeTypeId(HTMLFrameSetElementTypeId) => true,
                        _ => false
                    }
                })
            }
        }
    }

    // http://www.whatwg.org/specs/web-apps/current-work/#dom-document-body
    pub fn SetBody(&self, abstract_self: AbstractDocument, new_body: Option<AbstractNode>) -> ErrorResult {
        // Step 1.
        match new_body {
            Some(node) => {
                match node.type_id() {
                    ElementNodeTypeId(HTMLBodyElementTypeId) | ElementNodeTypeId(HTMLFrameSetElementTypeId) => {}
                    _ => return Err(HierarchyRequest)
                }
            }
            None => return Err(HierarchyRequest)
        }

        // Step 2.
        let old_body: Option<AbstractNode> = self.GetBody(abstract_self);
        if old_body == new_body {
            return Ok(());
        }

        // Step 3.
        match self.get_html_element() {
            // Step 4.
            None => return Err(HierarchyRequest),
            Some(root) => {
                match old_body {
                    Some(child) => { root.ReplaceChild(new_body.unwrap(), child); }
                    None => { root.AppendChild(new_body.unwrap()); }
                }
            }
        }
        Ok(())
    }

    pub fn GetElementsByName(&self, name: DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem|
            elem.get_attr(Null, "name").is_some() && eq_slice(elem.get_attr(Null, "name").unwrap(), name))
    }

    pub fn createHTMLCollection(&self, callback: |elem: &Element| -> bool) -> @mut HTMLCollection {
        let mut elements = ~[];
        match self.GetDocumentElement() {
            None => {},
            Some(root) => {
                for child in root.traverse_preorder() {
                    if child.is_element() {
                        child.with_imm_element(|elem| {
                            if callback(elem) {
                                elements.push(child);
                            }
                        });
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
        self.window.damage_and_reflow(damage);
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        self.window.wait_until_safe_to_modify_dom();
    }

    pub fn register_nodes_with_id(&mut self, root: &AbstractNode) {
        foreach_ided_elements(root, |id: &DOMString, abstract_node: &AbstractNode| {
            // TODO: "in tree order, within the context object's tree"
            // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
            self.idmap.find_or_insert(id.clone(), *abstract_node);
        });
    }

    pub fn unregister_nodes_with_id(&mut self, root: &AbstractNode) {
        foreach_ided_elements(root, |id: &DOMString, _| {
            // TODO: "in tree order, within the context object's tree"
            // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
            self.idmap.pop(id);
        });
    }

    pub fn update_idmap(&mut self,
                        abstract_self: AbstractNode,
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
                                  |_, new_node: AbstractNode| -> AbstractNode {
                                      new_node
                                  },
                                  |_, old_node: &mut AbstractNode, new_node: AbstractNode| {
                                      *old_node = new_node;
                                  });
            }
            None => ()
        }
    }
}

#[inline(always)]
fn foreach_ided_elements(root: &AbstractNode, callback: |&DOMString, &AbstractNode|) {
    for node in root.traverse_preorder() {
        if !node.is_element() {
            continue;
        }

        node.with_imm_element(|element| {
            match element.get_attr(Null, "id") {
                Some(id) => {
                    callback(&id.to_str(), &node);
                }
                None => ()
            }
        });
    }
}

impl Traceable for Document {
    fn trace(&self, tracer: *mut JSTracer) {
        self.node.trace(tracer);
    }
}
