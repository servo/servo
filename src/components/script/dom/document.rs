/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DocumentBinding;
use dom::bindings::utils::{DOMString, WrapperCache, ErrorResult, null_string, str};
use dom::bindings::utils::{BindingObject, CacheableWrapper, rust_box, DerivedWrapper};
use dom::element::{Element, HTMLHtmlElement};
use dom::element::{HTMLHtmlElementTypeId, HTMLHeadElementTypeId, HTMLTitleElementTypeId};
use dom::event::Event;
use dom::htmlcollection::HTMLCollection;
use dom::htmldocument::HTMLDocument;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, ScriptView, Node, ElementNodeTypeId, Text};
use dom::window::Window;
use dom::windowproxy::WindowProxy;
use dom::htmltitleelement::HTMLTitleElement;

use js::jsapi::{JS_AddObjectRoot, JS_RemoveObjectRoot, JSObject, JSContext, JSVal};
use js::glue::RUST_OBJECT_TO_JSVAL;
use servo_util::tree::TreeNodeRef;

use std::cast;
use std::ptr;
use std::str::eq_slice;

pub trait WrappableDocument {
    fn init_wrapper(@mut self, cx: *JSContext);
}

pub struct AbstractDocument {
    document: *Document
}

impl AbstractDocument {
    pub fn as_abstract<T: WrappableDocument>(cx: *JSContext, doc: @mut T) -> AbstractDocument {
        doc.init_wrapper(cx);
        AbstractDocument {
            document: unsafe { cast::transmute(doc) }
        }
    }

    unsafe fn transmute<T, R>(&self, f: &fn(&T) -> R) -> R {
        let box: *rust_box<T> = cast::transmute(self.document);
        f(&(*box).payload)
    }

    unsafe fn transmute_mut<T, R>(&self, f: &fn(&mut T) -> R) -> R {
        let box: *mut rust_box<T> = cast::transmute(self.document);
        f(&mut (*box).payload)
    }

    pub fn with_base<R>(&self, callback: &fn(&Document) -> R) -> R {
        unsafe {
            self.transmute(callback)
        }
    }

    pub fn with_mut_base<R>(&self, callback: &fn(&mut Document) -> R) -> R {
        unsafe {
            self.transmute_mut(callback)
        }
    }

    pub fn with_html<R>(&self, callback: &fn(&HTMLDocument) -> R) -> R {
        match self.with_base(|doc| doc.doctype) {
            HTML => unsafe { self.transmute(callback) },
            _ => fail!("attempt to downcast a non-HTMLDocument to HTMLDocument")
        }
    }
}

pub enum DocumentType {
    HTML,
    SVG,
    XML
}

pub struct Document {
    root: AbstractNode<ScriptView>,
    wrapper: WrapperCache,
    window: Option<@mut Window>,
    doctype: DocumentType,
    title: ~str
}

impl Document {
    pub fn new(root: AbstractNode<ScriptView>, window: Option<@mut Window>, doctype: DocumentType) -> Document {
        let compartment = unsafe {(*window.get_ref().page).js_info.get_ref().js_compartment };
        do root.with_base |base| {
            assert!(base.wrapper.get_wrapper().is_not_null());
            let rootable = base.wrapper.get_rootable();
            unsafe {
                JS_AddObjectRoot(compartment.cx.ptr, rootable);
            }
        }
        Document {
            root: root,
            wrapper: WrapperCache::new(),
            window: window,
            doctype: doctype,
            title: ~""
        }
    }

    pub fn Constructor(owner: @mut Window, _rv: &mut ErrorResult) -> AbstractDocument {
        let root = @HTMLHtmlElement {
            parent: HTMLElement::new(HTMLHtmlElementTypeId, ~"html")
        };

        let cx = unsafe {(*owner.page).js_info.get_ref().js_compartment.cx.ptr};
        let root = unsafe { Node::as_abstract_node(cx, root) };
        AbstractDocument::as_abstract(cx, @mut Document::new(root, None, XML))
    }
}

impl WrappableDocument for Document {
    fn init_wrapper(@mut self, cx: *JSContext) {
        self.wrap_object_shared(cx, ptr::null()); //XXXjdm a proper scope would be nice
    }
}

impl CacheableWrapper for AbstractDocument {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        do self.with_mut_base |doc| {
            doc.get_wrappercache()
        }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        match self.with_base(|doc| doc.doctype) {
            HTML => {
                let doc: @mut HTMLDocument = unsafe { cast::transmute(self.document) };
                doc.wrap_object_shared(cx, scope)
            }
            XML | SVG => {
                fail!("no wrapping for documents that don't exist")
            }
        }
    }
}

impl BindingObject for AbstractDocument {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        do self.with_mut_base |doc| {
            doc.GetParentObject(cx)
        }
    }
}

impl DerivedWrapper for AbstractDocument {
    fn wrap(&mut self, _cx: *JSContext, _scope: *JSObject, vp: *mut JSVal) -> i32 {
        let cache = self.get_wrappercache();
        let wrapper = cache.get_wrapper();
        unsafe { *vp = RUST_OBJECT_TO_JSVAL(wrapper) };
        return 1;
    }

    fn wrap_shared(@mut self, _cx: *JSContext, _scope: *JSObject, _vp: *mut JSVal) -> i32 {
        fail!(~"nyi")
    }
}


impl CacheableWrapper for Document {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe {
            cast::transmute(&self.wrapper)
        }
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        DocumentBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Document {
    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut CacheableWrapper> {
        match self.window {
            Some(win) => Some(win as @mut CacheableWrapper),
            None => None
        }
    }
}

impl Document {
    pub fn URL(&self) -> DOMString {
        null_string
    }

    pub fn DocumentURI(&self) -> DOMString {
        null_string
    }

    pub fn CompatMode(&self) -> DOMString {
        null_string
    }

    pub fn CharacterSet(&self) -> DOMString {
        null_string
    }

    pub fn ContentType(&self) -> DOMString {
        null_string
    }

    pub fn GetDocumentElement(&self) -> Option<AbstractNode<ScriptView>> {
        Some(self.root)
    }

    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let win = self.window.get_ref();
        let cx = unsafe {(*win.page).js_info.get_ref().js_compartment.cx.ptr};
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }

    pub fn GetElementsByTagName(&self, tag: &DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem| eq_slice(elem.tag_name, tag.to_str()))
    }

    pub fn GetElementsByTagNameNS(&self, _ns: &DOMString, _tag: &DOMString) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn GetElementsByClassName(&self, _class: &DOMString) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn GetElementById(&self, _id: &DOMString) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn CreateElement(&self, _local_name: &DOMString, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("stub")
    }

    pub fn CreateElementNS(&self, _namespace: &DOMString, _qualified_name: &DOMString, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("stub")
    }

    pub fn CreateEvent(&self, _interface: &DOMString, _rv: &mut ErrorResult) -> @mut Event {
        fail!("stub")
    }

    pub fn GetInputEncoding(&self) -> DOMString {
        null_string
    }

    pub fn Referrer(&self) -> DOMString {
        null_string
    }

    pub fn LastModified(&self) -> DOMString {
        null_string
    }

    pub fn ReadyState(&self) -> DOMString {
        null_string
    }

    pub fn Title(&self) -> DOMString {
        let mut title = ~"";
        match self.doctype {
            SVG => {
                fail!("no SVG document yet")
            },
            _ => {
                let _ = for node in self.root.traverse_preorder() {
                    if node.type_id() != ElementNodeTypeId(HTMLTitleElementTypeId) {
                        loop;
                    }
                    for child in node.children() {
                        if child.is_text() {
                            do child.with_imm_text() |text| {
                                let s = text.parent.Data();
                                title = title + s.to_str();
                            }
                        }
                    }
                    break;
                };
            }
        }
        let v: ~[&str] = title.word_iter().collect();
        title = v.connect(" ");
        title = title.trim().to_owned();
        str(title)
    }

    pub fn SetTitle(&self, title: &DOMString, _rv: &mut ErrorResult) {
        match self.doctype {
            SVG => {
                fail!("no SVG document yet")
            },
            _ => {
                let (_scope, cx) = self.get_scope_and_cx();
                let _ = for node in self.root.traverse_preorder() {
                    if node.type_id() != ElementNodeTypeId(HTMLHeadElementTypeId) {
                        loop;
                    }
                    let mut has_title = false;
                    for child in node.children() {
                        if child.type_id() != ElementNodeTypeId(HTMLTitleElementTypeId) {
                            loop;
                        }
                        has_title = true;
                        for title_child in child.children() {
                            child.remove_child(title_child);
                        }
                        let new_text = unsafe { 
                            Node::as_abstract_node(cx, @Text::new(title.to_str())) 
                        };
                        child.add_child(new_text);
                        break;
                    }
                    if !has_title {
                        let new_title = @HTMLTitleElement {
                            parent: HTMLElement::new(HTMLTitleElementTypeId, ~"title")
                        };
                        let new_title = unsafe { 
                            Node::as_abstract_node(cx, new_title) 
                        };
                        let new_text = unsafe {
                            Node::as_abstract_node(cx, @Text::new(title.to_str()))
                        };
                        new_title.add_child(new_text);
                        node.add_child(new_title);
                    }
                    break;
                };
            }
        }
    }

    pub fn Dir(&self) -> DOMString {
        null_string
    }

    pub fn SetDir(&self, _dir: &DOMString) {
    }

    pub fn GetDefaultView(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn GetActiveElement(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn HasFocus(&self, _rv: &mut ErrorResult) -> bool {
        false
    }

    pub fn GetCurrentScript(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn ReleaseCapture(&self) {
    }

    pub fn MozFullScreenEnabled(&self) -> bool {
        false
    }

    pub fn GetMozFullScreenElement(&self, _rv: &mut ErrorResult) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn GetMozPointerLockElement(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn MozExitPointerLock(&self) {
    }

    pub fn Hidden(&self) -> bool {
        false
    }

    pub fn MozHidden(&self) -> bool {
        self.Hidden()
    }

    pub fn VisibilityState(&self) -> DocumentBinding::VisibilityState {
        DocumentBinding::VisibilityStateValues::Visible
    }

    pub fn MozVisibilityState(&self) -> DocumentBinding::VisibilityState {
        self.VisibilityState()
    }

    pub fn GetSelectedStyleSheetSet(&self) -> DOMString {
        null_string
    }

    pub fn SetSelectedStyleSheetSet(&self, _sheet: &DOMString) {
    }

    pub fn GetLastStyleSheetSet(&self) -> DOMString {
        null_string
    }

    pub fn GetPreferredStyleSheetSet(&self) -> DOMString {
        null_string
    }

    pub fn EnableStyleSheetsForSet(&self, _name: &DOMString) {
    }

    pub fn ElementFromPoint(&self, _x: f32, _y: f32) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn QuerySelector(&self, _selectors: &DOMString, _rv: &mut ErrorResult) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn GetElementsByName(&self, name: &DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem|
            elem.get_attr("name").is_some() && eq_slice(elem.get_attr("name").unwrap(), name.to_str()))
    }
    
    pub fn createHTMLCollection(&self, callback: &fn(elem: &Element) -> bool) -> @mut HTMLCollection {
        let mut elements = ~[];
        let _ = for child in self.root.traverse_preorder() {
            if child.is_element() {
                do child.with_imm_element |elem| {
                    if callback(elem) {
                        elements.push(child);
                    }
                }
            }
        };
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(elements, cx, scope)
    }

    pub fn content_changed(&self) {
        for window in self.window.iter() {
            window.content_changed()
        }
    }

    pub fn teardown(&self) {
        unsafe {
            let compartment = (*self.window.get_ref().page).js_info.get_ref().js_compartment;
            do self.root.with_base |node| {
                assert!(node.wrapper.get_wrapper().is_not_null());
                let rootable = node.wrapper.get_rootable();
                JS_RemoveObjectRoot(compartment.cx.ptr, rootable);
            }
        }
    }
}

