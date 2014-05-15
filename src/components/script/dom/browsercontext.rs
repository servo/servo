/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, object_handle};
use dom::document::Document;
use dom::window::Window;

use js;
use js::jsapi::{JSObject, JS_PropertyStub, JS_DeletePropertyStub, JS_StrictPropertyStub};
use js::jsapi::{JS_EnumerateStub, JS_ResolveStub, JSFunctionSpec};
use js::glue::{WrapperNew, CreateWrapperProxyHandler, ProxyTraps};
use js::glue::{proxy_LookupGeneric, proxy_LookupProperty, proxy_LookupElement};
use js::glue::{proxy_DefineGeneric, proxy_DefineProperty, proxy_DefineElement};
use js::glue::{proxy_GetGeneric, proxy_SetGeneric, proxy_SetProperty, proxy_SetElement};
use js::glue::{proxy_GetGenericAttributes, proxy_SetGenericAttributes, proxy_DeleteProperty};
use js::glue::{proxy_DeleteElement, proxy_Trace, proxy_WeakmapKeyDelegate, proxy_Finalize};
use js::glue::{proxy_HasInstance, proxy_innerObject, proxy_Watch};
use js::glue::{proxy_Unwatch, proxy_Slice, proxy_Convert, proxy_GetProperty, proxy_GetElement};
use js::rust::with_compartment;

use libc;
use libc::c_void;
use std::ptr;

#[deriving(Encodable)]
pub struct BrowserContext {
    history: Vec<SessionHistoryEntry>,
    active_index: uint,
    window_proxy: Traceable<*mut JSObject>,
}

impl BrowserContext {
    pub fn new(document: &JSRef<Document>) -> BrowserContext {
        let mut context = BrowserContext {
            history: vec!(SessionHistoryEntry::new(document)),
            active_index: 0,
            window_proxy: Traceable::new(ptr::mut_null()),
        };
        context.window_proxy = Traceable::new(context.create_window_proxy());
        context
    }

    pub fn active_document(&self) -> Temporary<Document> {
        Temporary::new(self.history.get(self.active_index).document.clone())
    }

    pub fn active_window(&self) -> Temporary<Window> {
        let doc = self.active_document().root();
        Temporary::new(doc.deref().window.clone())
    }

    pub fn window_proxy(&self) -> *mut JSObject {
        assert!(self.window_proxy.deref().is_not_null());
        *self.window_proxy
    }

    pub fn create_window_proxy(&self) -> *mut JSObject {
        let win = self.active_window().root();
        let page = win.deref().page();
        let js_info = page.js_info();

        let handler = js_info.get_ref().dom_static.windowproxy_handler;
        assert!(handler.deref().is_not_null());

        let obj = win.deref().reflector().get_jsobject();
        let cx = js_info.get_ref().js_context.deref().deref().ptr;
        let wrapper = with_compartment(cx, obj, || unsafe {
            WrapperNew(cx, object_handle(&obj), object_handle(&obj), *handler.deref(),
                       &ProxyClass, true)
        });
        assert!(wrapper.is_not_null());
        wrapper
    }
}

static proxy_name: [u8, ..6] = ['P' as u8, 'r' as u8, 'o' as u8, 'x' as u8, 'y' as u8, 0];
static mut ProxyClass: js::Class = js::Class {
    name: &proxy_name as *u8 as *libc::c_char,
    flags: js::NON_NATIVE | js::JSCLASS_IS_PROXY | js::JSCLASS_IMPLEMENTS_BARRIERS |
           ((js::PROXY_MINIMUM_SLOTS & js::JSCLASS_RESERVED_SLOTS_MASK) << js::JSCLASS_RESERVED_SLOTS_SHIFT),
    addProperty: Some(JS_PropertyStub),
    delProperty: Some(JS_DeletePropertyStub),
    getProperty: Some(JS_PropertyStub),
    setProperty: Some(JS_StrictPropertyStub),
    enumerate: Some(JS_EnumerateStub),
    resolve: Some(JS_ResolveStub),
    convert: Some(proxy_Convert),
    finalize: Some(proxy_Finalize),
    call: None,
    hasInstance: Some(proxy_HasInstance),
    construct: None,
    trace: Some(proxy_Trace),

    spec: js::ClassSpec {
        createConstructor: None,
        createPrototype: None,
        constructorFunctions: 0 as *JSFunctionSpec,
        prototypeFunctions: 0 as *JSFunctionSpec,
        finishInit: None,
    },

    ext: js::ClassExtension {
        outerObject: None,
        innerObject: Some(proxy_innerObject),
        iteratorObject: 0 as *u8,
        isWrappedNative: 0,
        weakmapKeyDelegateOp: Some(proxy_WeakmapKeyDelegate),
    },

    ops: js::ObjectOps {
        lookupGeneric: Some(proxy_LookupGeneric),
        lookupProperty: Some(proxy_LookupProperty),
        lookupElement: Some(proxy_LookupElement),
        defineGeneric: Some(proxy_DefineGeneric),
        defineProperty: Some(proxy_DefineProperty),
        defineElement: Some(proxy_DefineElement),
        getGeneric: Some(proxy_GetGeneric),
        getProperty: Some(proxy_GetProperty),
        getElement: Some(proxy_GetElement),
        setGeneric: Some(proxy_SetGeneric),
        setProperty: Some(proxy_SetProperty),
        setElement: Some(proxy_SetElement),
        getGenericAttributes: Some(proxy_GetGenericAttributes),
        setGenericAttributes: Some(proxy_SetGenericAttributes),
        deleteProperty: Some(proxy_DeleteProperty),
        deleteElement: Some(proxy_DeleteElement),
        watch: Some(proxy_Watch),
        unwatch: Some(proxy_Unwatch),
        slice: Some(proxy_Slice),

        enumerate: 0 as *u8,
        thisObject: None,
    },
};

#[deriving(Encodable)]
pub struct SessionHistoryEntry {
    document: JS<Document>,
    children: Vec<BrowserContext>
}

impl SessionHistoryEntry {
    fn new(document: &JSRef<Document>) -> SessionHistoryEntry {
        SessionHistoryEntry {
            document: document.unrooted(),
            children: vec!()
        }
    }
}

static proxy_handler: ProxyTraps = ProxyTraps {
    preventExtensions: None,
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

    isExtensible: None,
    call: None,
    construct: None,
    nativeCall: 0 as *u8,
    hasInstance: None,
    objectClassIs: None,
    fun_toString: None,
    //regexp_toShared: 0 as *u8,
    defaultValue: None,
    finalize: None,
    getPrototypeOf: None,
    trace: None
};

pub fn new_window_proxy_handler() -> *c_void {
    unsafe {
        CreateWrapperProxyHandler(&proxy_handler)
    }
}
