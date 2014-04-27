/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JS;
use dom::bindings::trace::Traceable;
use dom::bindings::utils::Reflectable;
use dom::document::Document;
use dom::window::Window;

use js::jsapi::JSObject;
use js::glue::{WrapperNew, CreateWrapperProxyHandler, ProxyTraps};

use libc::c_void;
use std::ptr;

#[deriving(Encodable)]
pub struct BrowserContext {
    history: Vec<SessionHistoryEntry>,
    active_index: uint,
    window_proxy: Traceable<*JSObject>,
}

impl BrowserContext {
    pub fn new(document: &JS<Document>) -> BrowserContext {
        let mut context = BrowserContext {
            history: vec!(SessionHistoryEntry::new(document)),
            active_index: 0,
            window_proxy: Traceable::new(ptr::null()),
        };
        context.window_proxy = Traceable::new(context.create_window_proxy());
        context
    }

    pub fn active_document(&self) -> JS<Document> {
        self.history.get(self.active_index).document.clone()
    }

    pub fn active_window(&self) -> JS<Window> {
        let doc = self.active_document();
        doc.get().window.clone()
    }

    pub fn window_proxy(&self) -> *JSObject {
        assert!(self.window_proxy.deref().is_not_null());
        *self.window_proxy
    }

    pub fn create_window_proxy(&self) -> *JSObject {
        let win = self.active_window();
        let page = win.get().page();
        let js_info = page.js_info();

        let handler = js_info.get_ref().dom_static.windowproxy_handler;
        assert!(handler.deref().is_not_null());

        let parent = win.get().reflector().get_jsobject();
        let cx = js_info.get_ref().js_context.deref().deref().ptr;
        let wrapper = unsafe {
            WrapperNew(cx, parent, *handler.deref())
        };
        assert!(wrapper.is_not_null());
        wrapper
    }
}

#[deriving(Encodable)]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    children: Vec<BrowserContext>
}

impl SessionHistoryEntry {
    fn new(document: &JS<Document>) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: document.clone(),
            children: vec!()
        }
    }
}

static proxy_handler: ProxyTraps = ProxyTraps {
    getPropertyDescriptor: None,
    getOwnPropertyDescriptor: None,
    defineProperty: None,
    getOwnPropertyNames: 0 as *u8,
    delete_: None,
    enumerate: 0 as *u8,

    has: None,
    hasOwn: None,
    get: None,
    set: None,
    keys: 0 as *u8,
    iterate: None,

    call: None,
    construct: None,
    nativeCall: 0 as *u8,
    hasInstance: None,
    typeOf: None,
    objectClassIs: None,
    obj_toString: None,
    fun_toString: None,
    //regexp_toShared: 0 as *u8,
    defaultValue: None,
    iteratorNext: None,
    finalize: None,
    getElementIfPresent: None,
    getPrototypeOf: None,
    trace: None
};

pub fn new_window_proxy_handler() -> *c_void {
    unsafe {
        CreateWrapperProxyHandler(&proxy_handler)
    }
}
