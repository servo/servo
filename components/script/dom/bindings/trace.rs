/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Utilities for tracing JS-managed values.
//!
//! The lifetime of DOM objects is managed by the SpiderMonkey Garbage
//! Collector. A rooted DOM object implementing the interface `Foo` is traced
//! as follows:
//!
//! 1. The GC calls `_trace` defined in `FooBinding` during the marking
//!    phase. (This happens through `JSClass.trace` for non-proxy bindings, and
//!    through `ProxyTraps.trace` otherwise.)
//! 2. `_trace` calls `Foo::trace()` (an implementation of `JSTraceable`).
//!     This is typically derived via a #[jstraceable] annotation
//! 3. For all fields, `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `trace_object()` with the `JSObject` for the
//!    reflector.
//! 5. `trace_object()` calls `JS_CallTracer()` to notify the GC, which will
//!    add the object to the graph, and will trace that object as well.
//!
//! The untraceable!() macro adds an empty implementation of JSTraceable to
//! a datatype.

use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};
use js::jsapi::{JSObject, JSTracer, JS_CallTracer, JSTRACE_OBJECT};
use js::jsval::JSVal;

use libc;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

use url::Url;
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData};
use net::image_cache_task::ImageCacheTask;
use script_traits::ScriptControlChan;
use std::collections::hashmap::HashMap;
use collections::hash::Hash;
use style::PropertyDeclarationBlock;
use std::comm::{Receiver, Sender};
use hubbub::hubbub::QuirksMode;
use string_cache::{Atom, Namespace};
use js::rust::Cx;
use http::headers::response::HeaderCollection as ResponseHeaderCollection;
use http::headers::request::HeaderCollection as RequestHeaderCollection;
use http::method::Method;
use std::io::timer::Timer;
use script_traits::UntrustedNodeAddress;
use servo_msg::compositor_msg::ScriptListener;
use servo_msg::constellation_msg::ConstellationChan;
use servo_util::smallvec::{SmallVec1, SmallVec};
use layout_interface::{LayoutRPC, LayoutChan};
use dom::bindings::utils::WindowProxyHandler;

impl<T: Reflectable> JSTraceable for JS<T> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_reflector(trc, "", self.reflector());
    }
}

untraceable!(Reflector)

/// A trait to allow tracing (only) DOM objects.
pub trait JSTraceable {
    fn trace(&self, trc: *mut JSTracer);
}

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: JSVal) {
    if !val.is_markable() {
        return;
    }

    unsafe {
        let name = description.to_c_str();
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = -1;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing value {:s}", description);
        JS_CallTracer(tracer, val.to_gcthing(), val.trace_kind());
    }
}

/// Trace the `JSObject` held by `reflector`.
#[allow(unrooted_must_root)]
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    trace_object(tracer, description, reflector.get_jsobject())
}

/// Trace a `JSObject`.
pub fn trace_object(tracer: *mut JSTracer, description: &str, obj: *mut JSObject) {
    unsafe {
        let name = description.to_c_str();
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = -1;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing {:s}", description);
        JS_CallTracer(tracer, obj as *mut libc::c_void, JSTRACE_OBJECT);
    }
}

impl<T: JSTraceable> JSTraceable for RefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.borrow().trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Rc<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for Box<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable+Copy> JSTraceable for Cell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.get().trace(trc)
    }
}

impl JSTraceable for *mut JSObject {
    fn trace(&self, trc: *mut JSTracer) {
        trace_object(trc, "object", *self);
    }
}

impl JSTraceable for JSVal {
    fn trace(&self, trc: *mut JSTracer) {
        trace_jsval(trc, "val", *self);
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an untraceable type)
impl<T: JSTraceable> JSTraceable for Vec<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an untraceable type)
impl<T: JSTraceable + 'static> JSTraceable for SmallVec1<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

impl<T: JSTraceable> JSTraceable for Option<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        self.as_ref().map(|e| e.trace(trc));
    }
}

impl<K: Eq+Hash+JSTraceable, V: JSTraceable> JSTraceable for HashMap<K, V> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.val0().trace(trc);
            e.val1().trace(trc);
        }
    }
}

impl<A: JSTraceable, B: JSTraceable> JSTraceable for (A, B) {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        let (ref a, ref b) = *self;
        a.trace(trc);
        b.trace(trc);
    }
}


untraceable!(bool, f32, f64, String, Url)
untraceable!(uint, u8, u16, u32, u64)
untraceable!(int, i8, i16, i32, i64)
untraceable!(Sender<T>)
untraceable!(Receiver<T>)
untraceable!(ImageCacheTask, ScriptControlChan)
untraceable!(Atom, Namespace, Timer)
untraceable!(PropertyDeclarationBlock)
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
untraceable!(SubpageId, WindowSizeData, PipelineId)
untraceable!(QuirksMode)
untraceable!(Cx)
untraceable!(ResponseHeaderCollection, RequestHeaderCollection, Method)
untraceable!(ConstellationChan)
untraceable!(LayoutChan)
untraceable!(WindowProxyHandler)
untraceable!(UntrustedNodeAddress)

impl<'a> JSTraceable for &'a str {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl<A,B> JSTraceable for fn(A) -> B {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<ScriptListener+'static> {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<LayoutRPC+'static> {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}
