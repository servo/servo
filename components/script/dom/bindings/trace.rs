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
use dom::abstractworker::SharedRt;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::utils::WindowProxyHandler;
use encoding::types::EncodingRef;
use euclid::{Matrix2D, Matrix4D, Point2D};
use euclid::length::Length as EuclidLength;
use euclid::rect::Rect;
use euclid::size::Size2D;
use html5ever::tree_builder::QuirksMode;
use html5ever_atoms::{Prefix, LocalName, Namespace, QualName};
use hyper::header::Headers;
use hyper::method::Method;
use hyper::mime::Mime;
use hyper::status::StatusCode;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use js::glue::{CallObjectTracer, CallUnbarrieredObjectTracer, CallValueTracer};
use js::jsapi::{GCTraceKindToAscii, Heap, JSObject, JSTracer, TraceKind};
use js::jsval::JSVal;
use js::rust::Runtime;
use libc;
use msg::constellation_msg::{FrameId, FrameType, PipelineId};
use net_traits::{Metadata, NetworkError, ReferrerPolicy, ResourceThreads};
use net_traits::filemanager_thread::RelativePos;
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache_thread::{ImageCacheChan, ImageCacheThread};
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::response::HttpsState;
use net_traits::storage_thread::StorageType;
use offscreen_gl_context::GLLimits;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
use script_layout_interface::OpaqueStyleAndLayoutData;
use script_layout_interface::reporter::CSSErrorReporter;
use script_layout_interface::rpc::LayoutRPC;
use script_runtime::ScriptChan;
use script_traits::{TimerEventId, TimerSource, TouchpadPressurePhase};
use script_traits::{UntrustedNodeAddress, WindowSizeData, WindowSizeType};
use serde::{Deserialize, Serialize};
use servo_atoms::Atom;
use smallvec::SmallVec;
use std::boxed::FnBox;
use std::cell::{Cell, UnsafeCell};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{SystemTime, Instant};
use style::attr::{AttrIdentifier, AttrValue, LengthOrPercentageOrAuto};
use style::element_state::*;
use style::media_queries::MediaList;
use style::properties::PropertyDeclarationBlock;
use style::selector_impl::{PseudoElement, Snapshot};
use style::values::specified::Length;
use time::Duration;
use url::Origin as UrlOrigin;
use url::Url;
use uuid::Uuid;
use webrender_traits::{WebGLBufferId, WebGLError, WebGLFramebufferId, WebGLProgramId};
use webrender_traits::{WebGLRenderbufferId, WebGLShaderId, WebGLTextureId};

/// A trait to allow tracing (only) DOM objects.
pub trait JSTraceable {
    /// Trace `self`.
    fn trace(&self, trc: *mut JSTracer);
}

no_jsmanaged_fields!(CSSError);

no_jsmanaged_fields!(EncodingRef);

no_jsmanaged_fields!(Reflector);

no_jsmanaged_fields!(Duration);

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: &Heap<JSVal>) {
    unsafe {
        if !val.get().is_markable() {
            return;
        }

        debug!("tracing value {}", description);
        CallValueTracer(tracer,
                        val.ptr.get() as *mut _,
                        GCTraceKindToAscii(val.get().trace_kind()));
    }
}

/// Trace the `JSObject` held by `reflector`.
#[allow(unrooted_must_root)]
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    unsafe {
        debug!("tracing reflector {}", description);
        CallUnbarrieredObjectTracer(tracer,
                                    reflector.rootable(),
                                    GCTraceKindToAscii(TraceKind::Object));
    }
}

/// Trace a `JSObject`.
pub fn trace_object(tracer: *mut JSTracer, description: &str, obj: &Heap<*mut JSObject>) {
    unsafe {
        debug!("tracing {}", description);
        CallObjectTracer(tracer,
                         obj.ptr.get() as *mut _,
                         GCTraceKindToAscii(TraceKind::Object));
    }
}

impl<T: JSTraceable> JSTraceable for Rc<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

impl<T: JSTraceable + ?Sized> JSTraceable for Box<T> {
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

impl<T: JSTraceable> JSTraceable for DOMRefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        unsafe {
            (*self).borrow_for_gc_trace().trace(trc)
        }
    }
}

impl JSTraceable for Heap<*mut JSObject> {
    fn trace(&self, trc: *mut JSTracer) {
        if self.get().is_null() {
            return;
        }
        trace_object(trc, "heap object", self);
    }
}


impl JSTraceable for Heap<JSVal> {
    fn trace(&self, trc: *mut JSTracer) {
        trace_jsval(trc, "heap value", self);
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

impl<T: JSTraceable> JSTraceable for VecDeque<T> {
    #[inline]
    fn trace(&self, trc: *mut JSTracer) {
        for e in &*self {
            e.trace(trc);
        }
    }
}

impl<T: JSTraceable> JSTraceable for (T, T, T, T) {
    fn trace(&self, trc: *mut JSTracer) {
        self.0.trace(trc);
        self.1.trace(trc);
        self.2.trace(trc);
        self.3.trace(trc);
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

no_jsmanaged_fields!(bool, f32, f64, String, Url, AtomicBool, AtomicUsize, UrlOrigin, Uuid, char);
no_jsmanaged_fields!(usize, u8, u16, u32, u64);
no_jsmanaged_fields!(isize, i8, i16, i32, i64);
no_jsmanaged_fields!(Sender<T>);
no_jsmanaged_fields!(Receiver<T>);
no_jsmanaged_fields!(Point2D<T>);
no_jsmanaged_fields!(Rect<T>);
no_jsmanaged_fields!(Size2D<T>);
no_jsmanaged_fields!(Arc<T>);
no_jsmanaged_fields!(Image, ImageMetadata, ImageCacheChan, ImageCacheThread);
no_jsmanaged_fields!(Metadata);
no_jsmanaged_fields!(NetworkError);
no_jsmanaged_fields!(Atom, Prefix, LocalName, Namespace, QualName);
no_jsmanaged_fields!(Trusted<T: Reflectable>);
no_jsmanaged_fields!(TrustedPromise);
no_jsmanaged_fields!(PropertyDeclarationBlock);
no_jsmanaged_fields!(HashSet<T>);
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
no_jsmanaged_fields!(FrameId, FrameType, WindowSizeData, WindowSizeType, PipelineId);
no_jsmanaged_fields!(TimerEventId, TimerSource);
no_jsmanaged_fields!(WorkerId);
no_jsmanaged_fields!(QuirksMode);
no_jsmanaged_fields!(Runtime);
no_jsmanaged_fields!(Headers, Method);
no_jsmanaged_fields!(WindowProxyHandler);
no_jsmanaged_fields!(UntrustedNodeAddress);
no_jsmanaged_fields!(LengthOrPercentageOrAuto);
no_jsmanaged_fields!(RGBA);
no_jsmanaged_fields!(EuclidLength<Unit, T>);
no_jsmanaged_fields!(Matrix2D<T>);
no_jsmanaged_fields!(Matrix4D<T>);
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
no_jsmanaged_fields!(Snapshot);
no_jsmanaged_fields!(HttpsState);
no_jsmanaged_fields!(Request);
no_jsmanaged_fields!(SharedRt);
no_jsmanaged_fields!(TouchpadPressurePhase);
no_jsmanaged_fields!(USVString);
no_jsmanaged_fields!(ReferrerPolicy);
no_jsmanaged_fields!(Response);
no_jsmanaged_fields!(ResponseBody);
no_jsmanaged_fields!(ResourceThreads);
no_jsmanaged_fields!(StatusCode);
no_jsmanaged_fields!(SystemTime);
no_jsmanaged_fields!(Instant);
no_jsmanaged_fields!(RelativePos);
no_jsmanaged_fields!(OpaqueStyleAndLayoutData);
no_jsmanaged_fields!(PathBuf);
no_jsmanaged_fields!(CSSErrorReporter);
no_jsmanaged_fields!(WebGLBufferId);
no_jsmanaged_fields!(WebGLFramebufferId);
no_jsmanaged_fields!(WebGLProgramId);
no_jsmanaged_fields!(WebGLRenderbufferId);
no_jsmanaged_fields!(WebGLShaderId);
no_jsmanaged_fields!(WebGLTextureId);
no_jsmanaged_fields!(MediaList);

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
/// If you have GC things like *mut JSObject or JSVal, use rooted!.
/// If you have an arbitrary number of Reflectables to root, use rooted_vec!.
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

/// A vector of items to be rooted with `RootedVec`.
/// Guaranteed to be empty when not rooted.
/// Usage: `rooted_vec!(let mut v);` or if you have an
/// iterator of `Root`s, `rooted_vec!(let v <- iterator);`.
#[allow(unrooted_must_root)]
#[derive(JSTraceable)]
#[allow_unrooted_interior]
pub struct RootableVec<T: JSTraceable> {
    v: Vec<T>,
}

impl<T: JSTraceable> RootableVec<T> {
    /// Create a vector of items of type T that can be rooted later.
    pub fn new_unrooted() -> RootableVec<T> {
        RootableVec {
            v: vec![],
        }
   }
}

/// A vector of items that are rooted for the lifetime 'a.
#[allow_unrooted_interior]
pub struct RootedVec<'a, T: 'a + JSTraceable> {
    root: &'a mut RootableVec<T>,
}

impl<'a, T: JSTraceable + Reflectable> RootedVec<'a, JS<T>> {
    /// Create a vector of items of type T that is rooted for
    /// the lifetime of this struct
    pub fn new<I: Iterator<Item = Root<T>>>(root: &'a mut RootableVec<JS<T>>, iter: I)
                                            -> RootedVec<'a, JS<T>> {
        unsafe {
            RootedTraceableSet::add(root);
        }
        root.v.extend(iter.map(|item| JS::from_ref(&*item)));
        RootedVec {
            root: root,
        }
    }
}

impl<'a, T: JSTraceable> Drop for RootedVec<'a, T> {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            RootedTraceableSet::remove(self.root);
        }
    }
}

impl<'a, T: JSTraceable> Deref for RootedVec<'a, T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.root.v
    }
}

impl<'a, T: JSTraceable> DerefMut for RootedVec<'a, T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.root.v
    }
}

/// SM Callback that traces the rooted traceables
pub unsafe fn trace_traceables(tracer: *mut JSTracer) {
    debug!("tracing stack-rooted traceables");
    ROOTED_TRACEABLES.with(|ref traceables| {
        let traceables = traceables.borrow();
        traceables.trace(tracer);
    });
}
