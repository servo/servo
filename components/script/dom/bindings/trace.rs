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
//!    This is typically derived via a `#[dom_struct]` (implies `#[derive(JSTraceable)]`) annotation.
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

use canvas_traits::{CanvasGradientStop, LinearGradientStyle, RadialGradientStyle};
use canvas_traits::{LineCapStyle, LineJoinStyle, CompositionOrBlending, RepetitionStyle};
use cssparser::RGBA;
use encoding::types::EncodingRef;
use euclid::matrix2d::Matrix2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use html5ever::tree_builder::QuirksMode;
use hyper::header::Headers;
use hyper::method::Method;
use js::jsapi::{JSObject, JSTracer, JSGCTraceKind, JS_CallValueTracer, JS_CallObjectTracer, GCTraceKindToAscii, Heap};
use js::jsapi::JS_CallUnbarrieredObjectTracer;
use js::jsval::JSVal;
use js::rust::Runtime;
use layout_interface::{LayoutRPC, LayoutChan};
use libc;
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData, WorkerId};
use net_traits::image_cache_task::{ImageCacheChan, ImageCacheTask};
use net_traits::storage_task::StorageType;
use script_traits::ScriptControlChan;
use script_traits::UntrustedNodeAddress;
use smallvec::SmallVec1;
use msg::compositor_msg::ScriptListener;
use msg::constellation_msg::ConstellationChan;
use net_traits::image::base::Image;
use util::str::{LengthOrPercentageOrAuto};
use std::cell::{Cell, UnsafeCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::collections::hash_state::HashState;
use std::ffi::CString;
use std::hash::{Hash, Hasher};
use std::intrinsics::return_address;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
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

no_jsmanaged_fields!(EncodingRef);

no_jsmanaged_fields!(Reflector);

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: &Heap<JSVal>) {
    unsafe {
        if !val.get().is_markable() {
            return;
        }

        let name = CString::new(description).unwrap();
        (*tracer).debugPrinter_ = None;
        (*tracer).debugPrintIndex_ = !0;
        (*tracer).debugPrintArg_ = name.as_ptr() as *const libc::c_void;
        debug!("tracing value {}", description);
        JS_CallValueTracer(tracer, val.ptr.get() as *mut _,
                           GCTraceKindToAscii(val.get().trace_kind()));
    }
}

/// Trace the `JSObject` held by `reflector`.
#[allow(unrooted_must_root)]
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    unsafe {
        let name = CString::new(description).unwrap();
        (*tracer).debugPrinter_ = None;
        (*tracer).debugPrintIndex_ = !0;
        (*tracer).debugPrintArg_ = name.as_ptr() as *const libc::c_void;
        debug!("tracing reflector {}", description);
        JS_CallUnbarrieredObjectTracer(tracer, reflector.rootable(),
                                       GCTraceKindToAscii(JSGCTraceKind::JSTRACE_OBJECT));
    }
}

/// Trace a `JSObject`.
pub fn trace_object(tracer: *mut JSTracer, description: &str, obj: &Heap<*mut JSObject>) {
    unsafe {
        let name = CString::new(description).unwrap();
        (*tracer).debugPrinter_ = None;
        (*tracer).debugPrintIndex_ = !0;
        (*tracer).debugPrintArg_ = name.as_ptr() as *const libc::c_void;
        debug!("tracing {}", description);
        JS_CallObjectTracer(tracer, obj.ptr.get() as *mut _,
                            GCTraceKindToAscii(JSGCTraceKind::JSTRACE_OBJECT));
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

impl<T: JSTraceable> JSTraceable for *const T {
    fn trace(&self, trc: *mut JSTracer) {
        if !self.is_null() {
            unsafe {
                (**self).trace(trc)
            }
        }
    }
}

impl<T: JSTraceable> JSTraceable for *mut T {
    fn trace(&self, trc: *mut JSTracer) {
        if !self.is_null() {
            unsafe {
                (**self).trace(trc)
            }
        }
    }
}

impl<T: JSTraceable+Copy> JSTraceable for Cell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        self.get().trace(trc)
    }
}

impl<T: JSTraceable> JSTraceable for UnsafeCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        unsafe { (*self.get()).trace(trc) }
    }
}


impl JSTraceable for Heap<*mut JSObject> {
    fn trace(&self, trc: *mut JSTracer) {
        if self.get().is_null() {
            return;
        }
        trace_object(trc, "object", self);
    }
}


impl JSTraceable for Heap<JSVal> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_jsval(trc, "val", self);
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

impl<T: JSTraceable, U: JSTraceable> JSTraceable for Result<T, U> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        match *self {
            Ok(ref inner) => inner.trace(trc),
            Err(ref inner) => inner.trace(trc),
        }
    }
}

impl<K,V,S> JSTraceable for HashMap<K, V, S>
    where K: Hash + Eq + JSTraceable,
          V: JSTraceable,
          S: HashState,
          <S as HashState>::Hasher: Hasher,
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
no_jsmanaged_fields!(usize, u8, u16, u32, u64);
no_jsmanaged_fields!(isize, i8, i16, i32, i64);
no_jsmanaged_fields!(Sender<T>);
no_jsmanaged_fields!(Receiver<T>);
no_jsmanaged_fields!(Rect<T>);
no_jsmanaged_fields!(Size2D<T>);
no_jsmanaged_fields!(Arc<T>);
no_jsmanaged_fields!(Image, ImageCacheChan, ImageCacheTask, ScriptControlChan);
no_jsmanaged_fields!(Atom, Namespace);
no_jsmanaged_fields!(Trusted<T>);
no_jsmanaged_fields!(PropertyDeclarationBlock);
no_jsmanaged_fields!(HashSet<T>);
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
no_jsmanaged_fields!(SubpageId, WindowSizeData, PipelineId);
no_jsmanaged_fields!(WorkerId);
no_jsmanaged_fields!(QuirksMode);
no_jsmanaged_fields!(Runtime);
no_jsmanaged_fields!(Headers, Method);
no_jsmanaged_fields!(ConstellationChan);
no_jsmanaged_fields!(LayoutChan);
no_jsmanaged_fields!(WindowProxyHandler);
no_jsmanaged_fields!(UntrustedNodeAddress);
no_jsmanaged_fields!(LengthOrPercentageOrAuto);
no_jsmanaged_fields!(RGBA);
no_jsmanaged_fields!(Matrix2D<T>);
no_jsmanaged_fields!(StorageType);
no_jsmanaged_fields!(CanvasGradientStop, LinearGradientStyle, RadialGradientStyle);
no_jsmanaged_fields!(LineCapStyle, LineJoinStyle, CompositionOrBlending);
no_jsmanaged_fields!(RepetitionStyle);

impl JSTraceable for Box<ScriptChan+Send> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<Fn(f64, )> {
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

impl JSTraceable for () {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
    }
}

/// Homemade trait object for JSTraceable things
struct TraceableInfo {
    pub ptr: *const libc::c_void,
    pub trace: fn(obj: *const libc::c_void, tracer: *mut JSTracer)
}

/// Holds a set of JSTraceables that need to be rooted
pub struct RootedTraceableSet {
    set: Vec<TraceableInfo>
}

/// TLV Holds a set of JSTraceables that need to be rooted
thread_local!(pub static ROOTED_TRACEABLES: Rc<RefCell<RootedTraceableSet>> =
              Rc::new(RefCell::new(RootedTraceableSet::new())));

impl RootedTraceableSet {
    fn new() -> RootedTraceableSet {
        RootedTraceableSet {
            set: vec!()
        }
    }

    fn remove<T: JSTraceable>(traceable: &T) {
        ROOTED_TRACEABLES.with(|ref traceables| {
            let mut traceables = traceables.borrow_mut();
            let idx =
                match traceables.set.iter()
                                .rposition(|x| x.ptr == traceable as *const T as *const _) {
                    Some(idx) => idx,
                    None => unreachable!(),
                };
            traceables.set.remove(idx);
        });
    }

    fn add<T: JSTraceable>(traceable: &T) {
        ROOTED_TRACEABLES.with(|ref traceables| {
            fn trace<T: JSTraceable>(obj: *const libc::c_void, tracer: *mut JSTracer) {
                let obj: &T = unsafe { &*(obj as *const T) };
                obj.trace(tracer);
            }

            let mut traceables = traceables.borrow_mut();
            let info = TraceableInfo {
                ptr: traceable as *const T as *const libc::c_void,
                trace: trace::<T>,
            };
            traceables.set.push(info);
        })
    }

    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for info in self.set.iter() {
            (info.trace)(info.ptr, tracer);
        }
    }
}

/// Roots any JSTraceable thing
///
/// If you have a valid Reflectable, use Root.
/// If you have GC things like *mut JSObject or JSVal, use jsapi::Rooted.
/// If you have an arbitrary number of Reflectables to root, use RootedVec<JS<T>>
/// If you know what you're doing, use this.
#[derive(JSTraceable)]
pub struct RootedTraceable<'a, T: 'a + JSTraceable> {
    ptr: &'a T
}

impl<'a, T: JSTraceable> RootedTraceable<'a, T> {
    /// Root a JSTraceable thing for the life of this RootedTraceable
    pub fn new(traceable: &'a T) -> RootedTraceable<'a, T> {
        RootedTraceableSet::add(traceable);
        RootedTraceable { ptr: traceable }
    }
}

impl<'a, T: JSTraceable> Drop for RootedTraceable<'a, T> {
    fn drop(&mut self) {
        RootedTraceableSet::remove(self.ptr);
    }
}

/// A vector of items that are rooted for the lifetime
/// of this struct.
/// Must be a reflectable
#[allow(unrooted_must_root)]
#[no_move]
#[derive(JSTraceable)]
pub struct RootedVec<T: JSTraceable + Reflectable> {
    v: Vec<T>
}


impl<T: JSTraceable + Reflectable> RootedVec<T> {
    /// Create a vector of items of type T that is rooted for
    /// the lifetime of this struct
    pub fn new() -> RootedVec<T> {
        let addr = unsafe {
            return_address() as *const libc::c_void
        };

        RootedVec::new_with_destination_address(addr)
    }

    /// Create a vector of items of type T. This constructor is specific
    /// for RootTraceableSet.
    pub fn new_with_destination_address(addr: *const libc::c_void) -> RootedVec<T> {
        unsafe {
            RootedTraceableSet::add::<RootedVec<T>>(&*(addr as *const _));
        }
        RootedVec::<T> { v: vec!() }
    }
}

impl<T: JSTraceable + Reflectable> Drop for RootedVec<T> {
    fn drop(&mut self) {
        RootedTraceableSet::remove(self);
    }
}

impl<T: JSTraceable + Reflectable> Deref for RootedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.v
    }
}

impl<T: JSTraceable + Reflectable> DerefMut for RootedVec<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.v
    }
}

/// SM Callback that traces the rooted traceables
pub unsafe fn trace_traceables(tracer: *mut JSTracer) {
    ROOTED_TRACEABLES.with(|ref traceables| {
        let traceables = traceables.borrow();
        traceables.trace(tracer);
    });
}
