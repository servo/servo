/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JS;
use dom::bindings::proxyhandler::_obj_toString;
use dom::bindings::trace::trace_object;
use dom::bindings::utils::Reflectable;
use dom::document::Document;
use dom::window::Window;
use dom::windowproxy::WindowProxy;

use js::jsapi::{JSContext, JSObject, JSString, JSTracer};
use js::glue::{WrapperNew, CreateWrapperProxyHandler, ProxyTraps};

use std::cast;
use std::libc::c_void;
use std::ptr;
use serialize::{Encoder, Encodable};

#[deriving(Encodable)]
pub struct BrowserContext {
    history: ~[SessionHistoryEntry],
    active_index: uint,
    priv extra: Untraceable,
}

struct Untraceable {
    proxy_handler: *c_void,
    window_proxy: *JSObject,
}

impl<S: Encoder> Encodable<S> for Untraceable {
    fn encode(&self, tracer: &mut S) {
        let tracer: &mut JSTracer = unsafe { cast::transmute(tracer) };
        trace_object(tracer, "proxy", self.window_proxy);
    }
}

impl BrowserContext {
    pub fn new(document: JS<Document>) -> BrowserContext {
        let mut context = BrowserContext {
            history: ~[SessionHistoryEntry::new(document)],
            active_index: 0,
            extra: Untraceable {
                proxy_handler: new_window_proxy_handler(),
                window_proxy: ptr::null(),
            },
        };
        context.extra.window_proxy = context.create_window_proxy();
        context
    }

    pub fn active_document(&self) -> JS<Document> {
        self.history[self.active_index].document.clone()
    }

    pub fn active_window(&self) -> JS<Window> {
        let doc = self.active_document();
        doc.get().window.clone()
    }

    pub fn window_proxy(&self) -> WindowProxy {
        assert!(self.extra.window_proxy.is_not_null());
        self.extra.window_proxy
    }

    pub fn create_window_proxy(&self) -> WindowProxy {
        assert!(self.extra.proxy_handler.is_not_null());

        let win = self.active_window();
        let page = win.get().page();
        let js_info = page.js_info();

        let parent = win.get().reflector().get_jsobject();
        let cx = js_info.get_ref().js_context.deref().ptr;
        unsafe {
            WrapperNew(cx, parent, self.extra.proxy_handler)
        }
    }
}

#[deriving(Encodable)]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    children: ~[BrowserContext]
}

impl SessionHistoryEntry {
    fn new(document: JS<Document>) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: document,
            children: ~[]
        }
    }
}

extern fn obj_toString(cx: *JSContext, _proxy: *JSObject) -> *JSString {
    "Window".to_c_str().with_ref(|s| {
        _obj_toString(cx, s)
    })
}

fn new_window_proxy_handler() -> *c_void {
    let traps = ProxyTraps {
        getPropertyDescriptor: None,
        getOwnPropertyDescriptor: None,
        defineProperty: None,
        getOwnPropertyNames: ptr::null(),
        delete_: ptr::null(),
        enumerate: ptr::null(),

        has: None,
        hasOwn: None,
        get: None,
        set: ptr::null(),
        keys: ptr::null(),
        iterate: ptr::null(),

        call: ptr::null(),
        construct: ptr::null(),
        nativeCall: ptr::null(),
        hasInstance: ptr::null(),
        typeOf: ptr::null(),
        objectClassIs: ptr::null(),
        obj_toString: Some(obj_toString),
        fun_toString: ptr::null(),
        //regexp_toShared: ptr::null(),
        defaultValue: ptr::null(),
        iteratorNext: ptr::null(),
        finalize: None,
        getElementIfPresent: ptr::null(),
        getPrototypeOf: ptr::null(),
        trace: None
    };
    unsafe {
        CreateWrapperProxyHandler(&traps)
    }
}
