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
//!    This is typically derived via a `#[dom_struct]` (implies `#[jstraceable]`) annotation.
//!    Non-JS-managed types have an empty inline `trace()` method,
//!    achieved via `no_jsmanaged_fields!` or similar.
//! 3. For all fields, `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `trace_object()` with the `JSObject` for the
//!    reflector.
//! 5. `trace_object()` calls `JS_CallTracer()` to notify the GC, which will
//!    add the object to the graph, and will trace that object as well.
//! 6. When the GC finishes tracing, it [`finalizes`](../index.html#destruction)
//!    any reflectors that were not reachable.
//!
//! The `no_jsmanaged_fields!()` macro adds an empty implementation of `JSTraceable` to
//! a datatype.

use dom::bindings::js::JS;
use dom::bindings::refcounted::Trusted;
use dom::bindings::utils::{Reflectable, Reflector, WindowProxyHandler};
use script_task::ScriptChan;

use cssparser::RGBA;
use geom::rect::Rect;
use html5ever::tree_builder::QuirksMode;
use hyper::header::Headers;
use hyper::method::Method;
use js::jsapi::{JSObject, JSTracer, JS_CallTracer, JSGCTraceKind};
use js::jsval::JSVal;
use js::rust::Cx;
use layout_interface::{LayoutRPC, LayoutChan};
use libc;
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData};
use net::image_cache_task::ImageCacheTask;
use script_traits::ScriptControlChan;
use script_traits::UntrustedNodeAddress;
use msg::compositor_msg::ScriptListener;
use msg::constellation_msg::ConstellationChan;
use util::smallvec::{SmallVec1, SmallVec};
use util::str::{LengthOrPercentageOrAuto};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::collections::hash_state::HashState;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::old_io::timer::Timer;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use string_cache::{Atom, Namespace};
use style::properties::PropertyDeclarationBlock;
use url::Url;


/// A trait to allow tracing (only) DOM objects.
pub trait JSTraceable {
    /// Trace `self`.
    fn trace(&self, trc: *mut JSTracer);
}

impl<T: Reflectable> JSTraceable for JS<T> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_reflector(trc, "", self.reflector());
    }
}

no_jsmanaged_fields!(Reflector);

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: JSVal) {
    if !val.is_markable() {
        return;
    }

    unsafe {
        let name = CString::from_slice(description.as_bytes());
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = -1;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing value {}", description);
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
        let name = CString::from_slice(description.as_bytes());
        (*tracer).debugPrinter = None;
        (*tracer).debugPrintIndex = -1;
        (*tracer).debugPrintArg = name.as_ptr() as *const libc::c_void;
        debug!("tracing {}", description);
        JS_CallTracer(tracer, obj as *mut libc::c_void, JSGCTraceKind::JSTRACE_OBJECT);
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
// if e.trace() is a no-op (e.g it is an no_jsmanaged_fields type)
impl<T: JSTraceable> JSTraceable for Vec<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an no_jsmanaged_fields type)
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

impl<K,V,S> JSTraceable for HashMap<K, V, S>
    where K: Hash<<S as HashState>::Hasher> + Eq + JSTraceable,
          V: JSTraceable,
          S: HashState,
          <S as HashState>::Hasher: Hasher<Output=u64>,
{
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in self.iter() {
            k.trace(trc);
            v.trace(trc);
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


no_jsmanaged_fields!(bool, f32, f64, String, Url);
no_jsmanaged_fields!(uint, u8, u16, u32, u64);
no_jsmanaged_fields!(int, i8, i16, i32, i64);
no_jsmanaged_fields!(Sender<T>);
no_jsmanaged_fields!(Receiver<T>);
no_jsmanaged_fields!(Rect<T>);
no_jsmanaged_fields!(ImageCacheTask, ScriptControlChan);
no_jsmanaged_fields!(Atom, Namespace, Timer);
no_jsmanaged_fields!(Trusted<T>);
no_jsmanaged_fields!(PropertyDeclarationBlock);
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
no_jsmanaged_fields!(SubpageId, WindowSizeData, PipelineId);
no_jsmanaged_fields!(QuirksMode);
no_jsmanaged_fields!(Cx);
no_jsmanaged_fields!(Headers, Method);
no_jsmanaged_fields!(ConstellationChan);
no_jsmanaged_fields!(LayoutChan);
no_jsmanaged_fields!(WindowProxyHandler);
no_jsmanaged_fields!(UntrustedNodeAddress);
no_jsmanaged_fields!(LengthOrPercentageOrAuto);
no_jsmanaged_fields!(RGBA);

impl JSTraceable for Box<ScriptChan+Send> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

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
