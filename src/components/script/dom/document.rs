/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DocumentBinding;
use dom::bindings::utils::{DOMString, WrapperCache, ErrorResult, Fallible};
use dom::bindings::utils::{BindingObject, CacheableWrapper, DerivedWrapper};
use dom::bindings::utils::{is_valid_element_name, InvalidCharacter, Traceable, null_str_as_empty};
use dom::element::{Element};
use dom::element::{HTMLHtmlElementTypeId, HTMLHeadElementTypeId, HTMLTitleElementTypeId};
use dom::event::Event;
use dom::htmlcollection::HTMLCollection;
use dom::htmldocument::HTMLDocument;
use dom::htmlelement::HTMLElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::node::{AbstractNode, ScriptView, Node, ElementNodeTypeId};
use dom::text::Text;
use dom::window::Window;
use dom::windowproxy::WindowProxy;
use dom::htmltitleelement::HTMLTitleElement;
use html::hubbub_html_parser::build_element_from_tag;
use js::jsapi::{JSObject, JSContext, JSVal};
use js::jsapi::{JSTRACE_OBJECT, JSTracer, JS_CallTracer};
use js::glue::RUST_OBJECT_TO_JSVAL;
use servo_util::tree::TreeNodeRef;

use std::cast;
use std::ptr;
use std::str::eq_slice;
use std::libc;
use std::ascii::StrAsciiExt;
use std::unstable::raw::Box;

pub trait WrappableDocument {
    fn init_wrapper(@mut self, cx: *JSContext);
}

#[deriving(Eq)]
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
        let box: *Box<T> = cast::transmute(self.document);
        f(&(*box).data)
    }

    unsafe fn transmute_mut<T, R>(&self, f: &fn(&mut T) -> R) -> R {
        let box: *mut Box<T> = cast::transmute(self.document);
        f(&mut (*box).data)
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
    #[fixed_stack_segment]
    pub fn new(root: AbstractNode<ScriptView>, window: Option<@mut Window>, doctype: DocumentType) -> Document {
        Document {
            root: root,
            wrapper: WrapperCache::new(),
            window: window,
            doctype: doctype,
            title: ~""
        }
    }

    pub fn Constructor(owner: @mut Window) -> Fallible<AbstractDocument> {
        let root = @HTMLHtmlElement {
            htmlelement: HTMLElement::new(HTMLHtmlElementTypeId, ~"html")
        };

        let cx = owner.page.js_info.get_ref().js_compartment.cx.ptr;
        let root = unsafe { Node::as_abstract_node(cx, root) };
        Ok(AbstractDocument::as_abstract(cx, @mut Document::new(root, None, XML)))
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
    #[fixed_stack_segment]
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
        None
    }

    pub fn DocumentURI(&self) -> DOMString {
        None
    }

    pub fn CompatMode(&self) -> DOMString {
        None
    }

    pub fn CharacterSet(&self) -> DOMString {
        None
    }

    pub fn ContentType(&self) -> DOMString {
        None
    }

    pub fn GetDocumentElement(&self) -> Option<AbstractNode<ScriptView>> {
        Some(self.root)
    }

    fn get_cx(&self) -> *JSContext {
        let win = self.window.get_ref();
        win.page.js_info.get_ref().js_compartment.cx.ptr
    }

    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let win = self.window.get_ref();
        let cx = win.page.js_info.get_ref().js_compartment.cx.ptr;
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }

    pub fn GetElementsByTagName(&self, tag: &DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem| eq_slice(elem.tag_name, null_str_as_empty(tag)))
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

    pub fn CreateElement(&self, local_name: &DOMString) -> Fallible<AbstractNode<ScriptView>> {
        let cx = self.get_cx();
        let local_name = null_str_as_empty(local_name);
        if !is_valid_element_name(local_name) {
            return Err(InvalidCharacter);
        }
        let local_name = local_name.to_ascii_lower();
        Ok(build_element_from_tag(cx, local_name))
    }

    pub fn CreateElementNS(&self, _namespace: &DOMString, _qualified_name: &DOMString) -> Fallible<AbstractNode<ScriptView>> {
        fail!("stub")
    }

    pub fn CreateTextNode(&self, data: &DOMString) -> AbstractNode<ScriptView> {
        let cx = self.get_cx();
        unsafe { Node::as_abstract_node(cx, @Text::new(null_str_as_empty(data))) }
    }

    pub fn CreateEvent(&self, _interface: &DOMString) -> Fallible<@mut Event> {
        fail!("stub")
    }

    pub fn GetInputEncoding(&self) -> DOMString {
        None
    }

    pub fn Referrer(&self) -> DOMString {
        None
    }

    pub fn LastModified(&self) -> DOMString {
        None
    }

    pub fn ReadyState(&self) -> DOMString {
        None
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
                                let s = text.element.Data();
                                title = title + null_str_as_empty(&s);
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
        Some(title)
    }

    pub fn SetTitle(&self, title: &DOMString) -> ErrorResult {
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
                        child.add_child(self.CreateTextNode(title));
                        break;
                    }
                    if !has_title {
                        let new_title = @HTMLTitleElement {
                            htmlelement: HTMLElement::new(HTMLTitleElementTypeId, ~"title")
                        };
                        let new_title = unsafe { 
                            Node::as_abstract_node(cx, new_title) 
                        };
                        new_title.add_child(self.CreateTextNode(title));
                        node.add_child(new_title);
                    }
                    break;
                };
            }
        }
        Ok(())
    }

    pub fn Dir(&self) -> DOMString {
        None
    }

    pub fn SetDir(&self, _dir: &DOMString) {
    }

    pub fn GetDefaultView(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn GetActiveElement(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn HasFocus(&self) -> Fallible<bool> {
        Ok(false)
    }

    pub fn GetCurrentScript(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn ReleaseCapture(&self) {
    }

    pub fn MozFullScreenEnabled(&self) -> bool {
        false
    }

    pub fn GetMozFullScreenElement(&self) -> Fallible<Option<AbstractNode<ScriptView>>> {
        Ok(None)
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
        None
    }

    pub fn SetSelectedStyleSheetSet(&self, _sheet: &DOMString) {
    }

    pub fn GetLastStyleSheetSet(&self) -> DOMString {
        None
    }

    pub fn GetPreferredStyleSheetSet(&self) -> DOMString {
        None
    }

    pub fn EnableStyleSheetsForSet(&self, _name: &DOMString) {
    }

    pub fn ElementFromPoint(&self, _x: f32, _y: f32) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn QuerySelector(&self, _selectors: &DOMString) -> Fallible<Option<AbstractNode<ScriptView>>> {
        Ok(None)
    }

    pub fn GetElementsByName(&self, name: &DOMString) -> @mut HTMLCollection {
        self.createHTMLCollection(|elem|
            elem.get_attr("name").is_some() && eq_slice(elem.get_attr("name").unwrap(), null_str_as_empty(name)))
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

    pub fn wait_until_safe_to_modify_dom(&self) {
        for window in self.window.iter() {
            window.wait_until_safe_to_modify_dom();
        }
    }
}

impl Traceable for Document {
    #[fixed_stack_segment]
    fn trace(&self, tracer: *mut JSTracer) {
        unsafe {
            (*tracer).debugPrinter = ptr::null();
            (*tracer).debugPrintIndex = -1;
            do "root".to_c_str().with_ref |name| {
                (*tracer).debugPrintArg = name as *libc::c_void;
                debug!("tracing root node");
                do self.root.with_base |node| {
                    JS_CallTracer(tracer as *JSTracer,
                                  node.wrapper.wrapper,
                                  JSTRACE_OBJECT as u32);
                }
            }
        }
    }
}
