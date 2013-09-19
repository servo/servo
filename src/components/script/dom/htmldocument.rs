/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDocumentBinding;
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::bindings::utils::{CacheableWrapper, BindingObject, WrapperCache};
use dom::document::{AbstractDocument, Document, WrappableDocument, HTML};
use dom::element::HTMLHeadElementTypeId;
use dom::htmlcollection::HTMLCollection;
use dom::node::{AbstractNode, ScriptView, ElementNodeTypeId};
use dom::window::Window;

use js::jsapi::{JSObject, JSContext};

use servo_util::tree::TreeNodeRef;

use std::libc;
use std::ptr;
use std::str::eq_slice;

pub struct HTMLDocument {
    parent: Document
}

impl HTMLDocument {
    pub fn new(root: AbstractNode<ScriptView>, window: Option<@mut Window>) -> AbstractDocument {
        let doc = @mut HTMLDocument {
            parent: Document::new(root, window, HTML)
        };

        let compartment = window.get_ref().page.js_info.get_ref().js_compartment;
        AbstractDocument::as_abstract(compartment.cx.ptr, doc)
    }

    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let win = self.parent.window.get_ref();
        let cx = win.page.js_info.get_ref().js_compartment.cx.ptr;
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }
}

impl WrappableDocument for HTMLDocument {
    fn init_wrapper(@mut self, cx: *JSContext) {
        self.wrap_object_shared(cx, ptr::null()); //XXXjdm a proper scope would be nice
    }
}

impl HTMLDocument {
    pub fn NamedGetter(&self, _cx: *JSContext, _name: &DOMString, _found: &mut bool) -> Fallible<*JSObject> {
        Ok(ptr::null())
    }

    pub fn GetDomain(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn SetDomain(&self, _domain: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetCookie(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn SetCookie(&self, _cookie: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetHead(&self) -> Option<AbstractNode<ScriptView>> {
        let mut headNode: Option<AbstractNode<ScriptView>> = None;
        let _ = for child in self.parent.root.traverse_preorder() {
            if child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId) {
                headNode = Some(child);
                break;
            }
        };
        headNode 
    }

    pub fn Images(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "img"))
    }

    pub fn Embeds(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "embed"))
    }

    pub fn Plugins(&self) -> @mut HTMLCollection {
        self.Embeds()
    }

    pub fn Links(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem|
            (eq_slice(elem.tag_name, "a") || eq_slice(elem.tag_name, "area"))
            && elem.get_attr("href").is_some())
    }

    pub fn Forms(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "form"))
    }

    pub fn Scripts(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "script"))
    }

    pub fn Close(&self) -> ErrorResult {
        Ok(())
    }

    pub fn DesignMode(&self) -> DOMString {
        None
    }

    pub fn SetDesignMode(&self, _mode: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ExecCommand(&self, _command_id: &DOMString, _show_ui: bool, _value: &DOMString) -> Fallible<bool> {
        Ok(false)
    }

    pub fn QueryCommandEnabled(&self, _command_id: &DOMString) -> Fallible<bool> {
        Ok(false)
    }

    pub fn QueryCommandIndeterm(&self, _command_id: &DOMString) -> Fallible<bool> {
        Ok(false)
    }

    pub fn QueryCommandState(&self, _command_id: &DOMString) -> Fallible<bool> {
        Ok(false)
    }

    pub fn QueryCommandSupported(&self, _command_id: &DOMString) -> bool {
        false
    }

    pub fn QueryCommandValue(&self, _command_id: &DOMString) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn FgColor(&self) -> DOMString {
        None
    }

    pub fn SetFgColor(&self, _color: &DOMString) {
    }

    pub fn LinkColor(&self) -> DOMString {
        None
    }

    pub fn SetLinkColor(&self, _color: &DOMString) {
    }

    pub fn VlinkColor(&self) -> DOMString {
        None
    }

    pub fn SetVlinkColor(&self, _color: &DOMString) {
    }

    pub fn AlinkColor(&self) -> DOMString {
        None
    }

    pub fn SetAlinkColor(&self, _color: &DOMString) {
    }

    pub fn BgColor(&self) -> DOMString {
        None
    }

    pub fn SetBgColor(&self, _color: &DOMString) {
    }

    pub fn Anchors(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem|
            eq_slice(elem.tag_name, "a") && elem.get_attr("name").is_some())
    }

    pub fn Applets(&self) -> @mut HTMLCollection {
        // FIXME: This should be return OBJECT elements containing applets.
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "applet"))
    }

    pub fn Clear(&self) {
    }

    pub fn GetAll(&self, _cx: *JSContext) -> Fallible<*libc::c_void> {
        Ok(ptr::null())
    }
}

impl CacheableWrapper for HTMLDocument {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        self.parent.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        HTMLDocumentBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for HTMLDocument {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.parent.GetParentObject(cx)
    }
}

