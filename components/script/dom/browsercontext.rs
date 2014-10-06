/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, WindowProxyHandler};
use dom::document::Document;
use dom::window::Window;

use js::jsapi::JSObject;
use js::glue::{WrapperNew, CreateWrapperProxyHandler, ProxyTraps};
use js::rust::with_compartment;

use std::ptr;

#[allow(raw_pointer_deriving)]
#[jstraceable]
pub struct BrowserContext {
    history: Vec<SessionHistoryEntry>,
    active_index: uint,
    window_proxy: *mut JSObject,
}

impl BrowserContext {
    pub fn new(document: JSRef<Document>) -> BrowserContext {
        let mut context = BrowserContext {
            history: vec!(SessionHistoryEntry::new(document)),
            active_index: 0,
            window_proxy: ptr::null_mut(),
        };
        context.create_window_proxy();
        context
    }

    pub fn active_document(&self) -> Temporary<Document> {
        Temporary::new(self.history[self.active_index].document.clone())
    }

    pub fn active_window(&self) -> Temporary<Window> {
        let doc = self.active_document().root();
        Temporary::new(doc.deref().window.clone())
    }

    pub fn window_proxy(&self) -> *mut JSObject {
        assert!(self.window_proxy.is_not_null());
        self.window_proxy
    }

    fn create_window_proxy(&mut self) {
        let win = self.active_window().root();
        let page = win.deref().page();
        let js_info = page.js_info();

        let WindowProxyHandler(handler) = js_info.as_ref().unwrap().dom_static.windowproxy_handler;
        assert!(handler.is_not_null());

        let parent = win.deref().reflector().get_jsobject();
        let cx = js_info.as_ref().unwrap().js_context.deref().ptr;
        let wrapper = with_compartment(cx, parent, || unsafe {
            WrapperNew(cx, parent, handler)
        });
        assert!(wrapper.is_not_null());
        self.window_proxy = wrapper;
    }
}

#[jstraceable]
#[must_root]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    children: Vec<BrowserContext>
}

impl SessionHistoryEntry {
    fn new(document: JSRef<Document>) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: JS::from_rooted(document),
            children: vec!()
        }
    }
}

static proxy_handler: ProxyTraps = ProxyTraps {
    getPropertyDescriptor: None,
    getOwnPropertyDescriptor: None,
    defineProperty: None,
    getOwnPropertyNames: 0 as *const u8,
    delete_: None,
    enumerate: 0 as *const u8,

    has: None,
    hasOwn: None,
    get: None,
    set: None,
    keys: 0 as *const u8,
    iterate: None,

    call: None,
    construct: None,
    nativeCall: 0 as *const u8,
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

pub fn new_window_proxy_handler() -> WindowProxyHandler {
    unsafe {
        WindowProxyHandler(CreateWrapperProxyHandler(&proxy_handler))
    }
}
