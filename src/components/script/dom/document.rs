/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::comment::Comment;
use dom::bindings::codegen::DocumentBinding;
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::bindings::utils::{Reflectable, Reflector, DerivedWrapper};
use dom::bindings::utils::{is_valid_element_name, InvalidCharacter, Traceable, null_str_as_empty, null_str_as_word_null};
use dom::documentfragment::DocumentFragment;
use dom::element::{Element};
use dom::element::{HTMLHeadElementTypeId, HTMLTitleElementTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::htmldocument::HTMLDocument;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView, Node, ElementNodeTypeId, DocumentNodeTypeId};
use dom::text::Text;
use dom::window::Window;
use dom::htmltitleelement::HTMLTitleElement;
use html::hubbub_html_parser::build_element_from_tag;
use js::jsapi::{JSObject, JSContext, JSVal, JSTracer};
use js::glue::RUST_OBJECT_TO_JSVAL;
use servo_util::tree::{TreeNodeRef, ElementLike};

use std::hashmap::HashMap;

use std::cast;
use std::ptr;
use std::str::eq_slice;
use std::ascii::StrAsciiExt;
use std::unstable::raw::Box;

#[deriving(Eq)]
pub enum DocumentTypeId {
    PlainDocumentTypeId,
    HTMLDocumentTypeId
}

pub trait ReflectableDocument {
    fn init_reflector(@mut self, cx: *JSContext);
    fn init_node(@mut self, doc: AbstractDocument);
}

#[deriving(Eq)]
pub struct AbstractDocument {
    document: *mut Box<Document>
}

impl AbstractDocument {
    pub fn as_abstract<T: ReflectableDocument>(cx: *JSContext, doc: @mut T) -> AbstractDocument {
        doc.init_reflector(cx);
        let abstract = AbstractDocument {
            document: unsafe { cast::transmute(doc) }
        };
        doc.init_node(abstract);
        abstract
    }

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

    unsafe fn transmute<T, R>(&self, f: &fn(&T) -> R) -> R {
        let box: *Box<T> = cast::transmute(self.document);
        f(&(*box).data)
    }

    unsafe fn transmute_mut<T, R>(&self, f: &fn(&mut T) -> R) -> R {
        let box: *mut Box<T> = cast::transmute(self.document);
        f(&mut (*box).data)
    }

    pub fn with_html<R>(&self, callback: &fn(&HTMLDocument) -> R) -> R {
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

    pub fn set_root(&self, root: AbstractNode<ScriptView>) {
        assert!(do root.traverse_preorder().all |node| {
            node.node().owner_doc() == *self
        });

        let document = self.mut_document();
        document.node.AppendChild(AbstractNode::from_document(*self), root);
        // Register elements having "id" attribute to the owner doc.
        document.register_nodes_with_id(&root);
    }
}

pub enum DocumentType {
    HTML,
    SVG,
    XML
}

pub struct Document {
    node: Node<ScriptView>,
    reflector_: Reflector,
    window: @mut Window,
    doctype: DocumentType,
    title: ~str,
    idmap: HashMap<~str, AbstractNode<ScriptView>>
}

impl Document {
    #[fixed_stack_segment]
    pub fn new(window: @mut Window, doctype: DocumentType) -> Document {
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

    pub fn Constructor(owner: @mut Window) -> Fallible<AbstractDocument> {
        let cx = owner.get_cx();
        Ok(AbstractDocument::as_abstract(cx, @mut Document::new(owner, XML)))
    }
}

impl ReflectableDocument for Document {
    fn init_reflector(@mut self, cx: *JSContext) {
        self.wrap_object_shared(cx, ptr::null()); //XXXjdm a proper scope would be nice
    }

    fn init_node(@mut self, doc: AbstractDocument) {
        self.node.set_owner_doc(doc);
    }
}

impl Reflectable for AbstractDocument {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.document().reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.mut_document().mut_reflector()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        match self.document().doctype {
            HTML => {
                let doc: @mut HTMLDocument = unsafe { cast::transmute(self.document) };
                doc.wrap_object_shared(cx, scope)
            }
            XML | SVG => {
                fail!("no wrapping for documents that don't exist")
            }
        }
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        self.document().GetParentObject(cx)
    }
}

impl DerivedWrapper for AbstractDocument {
    #[fixed_stack_segment]
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, vp: *mut JSVal) -> i32 {
        unsafe { *vp = RUST_OBJECT_TO_JSVAL(self.reflector().get_jsobject()) };
        return 1;
    }
}


impl Reflectable for Document {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        DocumentBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut Reflectable> {
        Some(self.window as @mut Reflectable)
    }
}

impl Document {
    pub fn GetDocumentElement(&self) -> Option<AbstractNode<ScriptView>> {
        self.node.first_child
    }

    fn get_cx(&self) -> *JSContext {
        self.window.get_cx()
    }

    pub fn GetElementsByTagName(&self, tag: &DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem| eq_slice(elem.tag_name, null_str_as_empty(tag)))
    }

    pub fn GetElementsByTagNameNS(&self, _ns: &DOMString, _tag: &DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.window, ~[])
    }

    pub fn GetElementsByClassName(&self, _class: &DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.window, ~[])
    }

    pub fn GetElementById(&self, id: &DOMString) -> Option<AbstractNode<ScriptView>> {
        let key: &~str = &null_str_as_empty(id);
        // TODO: "in tree order, within the context object's tree"
        // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
        match self.idmap.find_equiv(key) {
            None => None,
            Some(node) => Some(*node),
        }
    }

    pub fn CreateElement(&self, abstract_self: AbstractDocument, local_name: &DOMString) -> Fallible<AbstractNode<ScriptView>> {
        let cx = self.get_cx();
        let local_name = null_str_as_empty(local_name);
        if !is_valid_element_name(local_name) {
            return Err(InvalidCharacter);
        }
        let local_name = local_name.to_ascii_lower();
        Ok(build_element_from_tag(cx, local_name, abstract_self))
    }

    pub fn CreateDocumentFragment(&self, abstract_self: AbstractDocument) -> AbstractNode<ScriptView> {
        let cx = self.get_cx();
        let fragment = @DocumentFragment::new(abstract_self);
        unsafe { Node::as_abstract_node(cx, fragment) }
    }

    pub fn CreateTextNode(&self, abstract_self: AbstractDocument, data: &DOMString) -> AbstractNode<ScriptView> {
        let cx = self.get_cx();
        let text = @Text::new(null_str_as_empty(data), abstract_self);
        unsafe { Node::as_abstract_node(cx, text) }
    }

    pub fn CreateComment(&self, abstract_self: AbstractDocument, data: &DOMString) -> AbstractNode<ScriptView> {
        let cx = self.get_cx();
        let comment = @Comment::new(null_str_as_word_null(data), abstract_self);
        unsafe { Node::as_abstract_node(cx, comment) }
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
                                    do child.with_imm_text() |text| {
                                        let s = text.element.Data();
                                        title = title + null_str_as_empty(&s);
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        let v: ~[&str] = title.word_iter().collect();
        title = v.connect(" ");
        title = title.trim().to_owned();
        Some(title)
    }

    pub fn SetTitle(&self, abstract_self: AbstractDocument, title: &DOMString) -> ErrorResult {
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
                                    child.remove_child(title_child);
                                }
                                child.AppendChild(self.CreateTextNode(abstract_self, title));
                                break;
                            }
                            if !has_title {
                                let new_title = @HTMLTitleElement {
                                    htmlelement: HTMLElement::new(HTMLTitleElementTypeId, ~"title", abstract_self)
                                };
                                let new_title = unsafe { 
                                    Node::as_abstract_node(self.get_cx(), new_title)
                                };
                                new_title.AppendChild(self.CreateTextNode(abstract_self, title));
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

    pub fn GetElementsByName(&self, name: &DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem|
            elem.get_attr("name").is_some() && eq_slice(elem.get_attr("name").unwrap(), null_str_as_empty(name)))
    }

    pub fn createHTMLCollection(&self, callback: &fn(elem: &Element) -> bool) -> @mut HTMLCollection {
        let mut elements = ~[];
        match self.GetDocumentElement() {
            None => {},
            Some(root) => {
                for child in root.traverse_preorder() {
                    if child.is_element() {
                        do child.with_imm_element |elem| {
                            if callback(elem) {
                                elements.push(child);
                            }
                        }
                    }
                }
            }
        }
        HTMLCollection::new(self.window, elements)
    }

    pub fn content_changed(&self) {
        self.window.content_changed();
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        self.window.wait_until_safe_to_modify_dom();
    }

    pub fn register_nodes_with_id(&mut self, root: &AbstractNode<ScriptView>) {
        foreach_ided_elements(root, |id: &~str, abstract_node: &AbstractNode<ScriptView>| {
            // TODO: "in tree order, within the context object's tree"
            // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
            self.idmap.find_or_insert(id.clone(), *abstract_node);
        });
    }

    pub fn unregister_nodes_with_id(&mut self, root: &AbstractNode<ScriptView>) {
        foreach_ided_elements(root, |id: &~str, _| {
            // TODO: "in tree order, within the context object's tree"
            // http://dom.spec.whatwg.org/#dom-document-getelementbyid.
            self.idmap.pop(id);
        });
    }
}

#[inline(always)]
fn foreach_ided_elements(root: &AbstractNode<ScriptView>,
                         callback: &fn(&~str, &AbstractNode<ScriptView>)) {
    for node in root.traverse_preorder() {
        if !node.is_element() {
            continue;
        }

        do node.with_imm_element |element| {
            match element.get_attr("id") {
                Some(id) => {
                    callback(&id.to_str(), &node);
                }
                None => ()
            }
        }
    }
}

impl Traceable for Document {
    #[fixed_stack_segment]
    fn trace(&self, tracer: *mut JSTracer) {
        self.node.trace(tracer);
    }
}
