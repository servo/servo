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
//!    This is typically derived via a `#[dom_struct]`
//!    (implies `#[derive(JSTraceable)]`) annotation.
//!    Non-JS-managed types have an empty inline `trace()` method,
//!    achieved via `no_jsmanaged_fields!` or similar.
//! 3. For all fields, `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `JS_CallUnbarrieredObjectTracer()` with a
//!    pointer to the `JSObject` for the reflector. This notifies the GC, which
//!    will add the object to the graph, and will trace that object as well.
//! 5. When the GC finishes tracing, it [`finalizes`](../index.html#destruction)
//!    any reflectors that were not reachable.
//!
//! The `no_jsmanaged_fields!()` macro adds an empty implementation of `JSTraceable` to
//! a datatype.

use canvas_traits::{CanvasGradientStop, LinearGradientStyle, RadialGradientStyle};
use canvas_traits::{CompositionOrBlending, LineCapStyle, LineJoinStyle, RepetitionStyle};
use cssparser::RGBA;
use devtools_traits::CSSError;
use devtools_traits::WorkerId;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::bindings::utils::WindowProxyHandler;
use dom::worker::SharedRt;
use encoding::types::EncodingRef;
use euclid::length::Length as EuclidLength;
use euclid::matrix2d::Matrix2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use html5ever::tree_builder::QuirksMode;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::mime::Mime;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use js::jsapi::JS_CallUnbarrieredObjectTracer;
use js::jsapi::{GCTraceKindToAscii, Heap, JSGCTraceKind, JSObject, JSTracer, JS_CallObjectTracer, JS_CallValueTracer};
use js::jsval::JSVal;
use js::rust::Runtime;
use layout_interface::{LayoutChan, LayoutRPC};
use libc;
use msg::constellation_msg::ConstellationChan;
use msg::constellation_msg::{PipelineId, SubpageId, WindowSizeData};
use net_traits::Metadata;
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheThread};
use net_traits::response::HttpsState;
use net_traits::storage_thread::StorageType;
use offscreen_gl_context::GLLimits;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
use script_runtime::ScriptChan;
use script_traits::{LayoutMsg, ScriptMsg, TimerEventId, TimerSource, TouchpadPressurePhase, UntrustedNodeAddress};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::boxed::FnBox;
use std::cell::{Cell, UnsafeCell};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::CString;
use std::hash::{BuildHasher, Hash};
use std::intrinsics::return_address;
use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::{Receiver, Sender};
use string_cache::{Atom, Namespace, QualName};
use style::attr::{AttrIdentifier, AttrValue};
use style::element_state::*;
use style::properties::PropertyDeclarationBlock;
use style::restyle_hints::ElementSnapshot;
use style::selector_impl::PseudoElement;
use style::values::specified::Length;
use url::Origin as UrlOrigin;
use url::Url;
use util::str::{DOMString, LengthOrPercentageOrAuto};
use uuid::Uuid;
use webrender_traits::WebGLError;

/// A trait to allow tracing (only) DOM objects.
pub trait JSTraceable {
    /// Trace `self`.
    fn trace(&self, trc: *mut JSTracer);
}

no_jsmanaged_fields!(CSSError);

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
        JS_CallValueTracer(tracer,
                           val.ptr.get() as *mut _,
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
        JS_CallUnbarrieredObjectTracer(tracer,
                                       reflector.rootable(),
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
        JS_CallObjectTracer(tracer,
                            obj.ptr.get() as *mut _,
                            GCTraceKindToAscii(JSGCTraceKind::JSTRACE_OBJECT));
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

impl<T: JSTraceable + Copy> JSTraceable for Cell<T> {
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
        for e in &*self {
            e.trace(trc);
        }
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an no_jsmanaged_fields type)
impl<T: JSTraceable + 'static> JSTraceable for SmallVec<[T; 1]> {
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

impl<K, V, S> JSTraceable for HashMap<K, V, S>
    where K: Hash + Eq + JSTraceable,
          V: JSTraceable,
          S: BuildHasher
{
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in &*self {
            k.trace(trc);
            v.trace(trc);
        }
    }
}

impl<K: Ord + JSTraceable, V: JSTraceable> JSTraceable for BTreeMap<K, V> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in self {
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

impl<A: JSTraceable, B: JSTraceable, C: JSTraceable> JSTraceable for (A, B, C) {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        let (ref a, ref b, ref c) = *self;
        a.trace(trc);
        b.trace(trc);
        c.trace(trc);
    }
}

no_jsmanaged_fields!(bool, f32, f64, String, Url, AtomicBool, AtomicUsize, UrlOrigin, Uuid);
no_jsmanaged_fields!(usize, u8, u16, u32, u64);
no_jsmanaged_fields!(isize, i8, i16, i32, i64);
no_jsmanaged_fields!(Sender<T>);
no_jsmanaged_fields!(Receiver<T>);
no_jsmanaged_fields!(Rect<T>);
no_jsmanaged_fields!(Size2D<T>);
no_jsmanaged_fields!(Arc<T>);
no_jsmanaged_fields!(Image, ImageMetadata, ImageCacheChan, ImageCacheThread);
no_jsmanaged_fields!(Metadata);
no_jsmanaged_fields!(Atom, Namespace, QualName);
no_jsmanaged_fields!(Trusted<T: Reflectable>);
no_jsmanaged_fields!(PropertyDeclarationBlock);
no_jsmanaged_fields!(HashSet<T>);
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
no_jsmanaged_fields!(SubpageId, WindowSizeData, PipelineId);
no_jsmanaged_fields!(TimerEventId, TimerSource);
no_jsmanaged_fields!(WorkerId);
no_jsmanaged_fields!(QuirksMode);
no_jsmanaged_fields!(Runtime);
no_jsmanaged_fields!(Headers, Method);
no_jsmanaged_fields!(LayoutChan);
no_jsmanaged_fields!(WindowProxyHandler);
no_jsmanaged_fields!(UntrustedNodeAddress);
no_jsmanaged_fields!(LengthOrPercentageOrAuto);
no_jsmanaged_fields!(RGBA);
no_jsmanaged_fields!(EuclidLength<Unit, T>);
no_jsmanaged_fields!(Matrix2D<T>);
no_jsmanaged_fields!(PhantomData<T>);
no_jsmanaged_fields!(StorageType);
no_jsmanaged_fields!(CanvasGradientStop, LinearGradientStyle, RadialGradientStyle);
no_jsmanaged_fields!(LineCapStyle, LineJoinStyle, CompositionOrBlending);
no_jsmanaged_fields!(RepetitionStyle);
no_jsmanaged_fields!(WebGLError, GLLimits);
no_jsmanaged_fields!(TimeProfilerChan);
no_jsmanaged_fields!(MemProfilerChan);
no_jsmanaged_fields!(PseudoElement);
no_jsmanaged_fields!(Length);
no_jsmanaged_fields!(ElementState);
no_jsmanaged_fields!(DOMString);
no_jsmanaged_fields!(Mime);
no_jsmanaged_fields!(AttrIdentifier);
no_jsmanaged_fields!(AttrValue);
no_jsmanaged_fields!(ElementSnapshot);
no_jsmanaged_fields!(HttpsState);
no_jsmanaged_fields!(SharedRt);
no_jsmanaged_fields!(TouchpadPressurePhase);

impl JSTraceable for ConstellationChan<ScriptMsg> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for ConstellationChan<LayoutMsg> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<ScriptChan + Send> {
    #[inline]
    fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<FnBox(f64, )> {
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

impl<A, B> JSTraceable for fn(A) -> B {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl<T> JSTraceable for IpcSender<T> where T: Deserialize + Serialize {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for Box<LayoutRPC + 'static> {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl JSTraceable for () {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

impl<T> JSTraceable for IpcReceiver<T> where T: Deserialize + Serialize {
    #[inline]
    fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

/// Homemade trait object for JSTraceable things
struct TraceableInfo {
    pub ptr: *const libc::c_void,
    pub trace: fn(obj: *const libc::c_void, tracer: *mut JSTracer),
}

/// Holds a set of JSTraceables that need to be rooted
pub struct RootedTraceableSet {
    set: Vec<TraceableInfo>,
}

#[allow(missing_docs)]  // FIXME
mod dummy {  // Attributes don’t apply through the macro.
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::RootedTraceableSet;
    /// TLV Holds a set of JSTraceables that need to be rooted
    thread_local!(pub static ROOTED_TRACEABLES: Rc<RefCell<RootedTraceableSet>> =
                  Rc::new(RefCell::new(RootedTraceableSet::new())));
}
pub use self::dummy::ROOTED_TRACEABLES;

impl RootedTraceableSet {
    fn new() -> RootedTraceableSet {
        RootedTraceableSet {
            set: vec![],
        }
    }

    unsafe fn remove<T: JSTraceable>(traceable: &T) {
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

    unsafe fn add<T: JSTraceable>(traceable: &T) {
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
        for info in &self.set {
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
    ptr: &'a T,
}

impl<'a, T: JSTraceable> RootedTraceable<'a, T> {
    /// Root a JSTraceable thing for the life of this RootedTraceable
    pub fn new(traceable: &'a T) -> RootedTraceable<'a, T> {
        unsafe {
            RootedTraceableSet::add(traceable);
        }
        RootedTraceable {
            ptr: traceable,
        }
    }
}

impl<'a, T: JSTraceable> Drop for RootedTraceable<'a, T> {
    fn drop(&mut self) {
        unsafe {
            RootedTraceableSet::remove(self.ptr);
        }
    }
}

/// A vector of items that are rooted for the lifetime of this struct.
#[allow(unrooted_must_root)]
#[no_move]
#[derive(JSTraceable)]
#[allow_unrooted_interior]
pub struct RootedVec<T: JSTraceable> {
    v: Vec<T>,
}


impl<T: JSTraceable> RootedVec<T> {
    /// Create a vector of items of type T that is rooted for
    /// the lifetime of this struct
    pub fn new() -> RootedVec<T> {
        let addr = unsafe { return_address() as *const libc::c_void };

        unsafe { RootedVec::new_with_destination_address(addr) }
    }

    /// Create a vector of items of type T. This constructor is specific
    /// for RootTraceableSet.
    pub unsafe fn new_with_destination_address(addr: *const libc::c_void) -> RootedVec<T> {
        RootedTraceableSet::add::<RootedVec<T>>(&*(addr as *const _));
        RootedVec::<T> {
            v: vec![],
        }
    }
}

impl<T: JSTraceable + Reflectable> RootedVec<JS<T>> {
    /// Obtain a safe slice of references that can't outlive that RootedVec.
    pub fn r(&self) -> &[&T] {
        unsafe { mem::transmute(&self.v[..]) }
    }
}

impl<T: JSTraceable> Drop for RootedVec<T> {
    fn drop(&mut self) {
        unsafe {
            RootedTraceableSet::remove(self);
        }
    }
}

impl<T: JSTraceable> Deref for RootedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.v
    }
}

impl<T: JSTraceable> DerefMut for RootedVec<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.v
    }
}

impl<A: JSTraceable + Reflectable> FromIterator<Root<A>> for RootedVec<JS<A>> {
    #[allow(moved_no_move)]
    fn from_iter<T>(iterable: T) -> RootedVec<JS<A>>
        where T: IntoIterator<Item = Root<A>>
    {
        let mut vec = unsafe {
            RootedVec::new_with_destination_address(return_address() as *const libc::c_void)
        };
        vec.extend(iterable.into_iter().map(|item| JS::from_rooted(&item)));
        vec
    }
}

/// SM Callback that traces the rooted traceables
pub unsafe fn trace_traceables(tracer: *mut JSTracer) {
    ROOTED_TRACEABLES.with(|ref traceables| {
        let traceables = traceables.borrow();
        traceables.trace(tracer);
    });
}
