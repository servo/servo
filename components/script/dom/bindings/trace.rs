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
//!    achieved via `unsafe_no_jsmanaged_fields!` or similar.
//! 3. For all fields, `Foo::trace()`
//!    calls `trace()` on the field.
//!    For example, for fields of type `JS<T>`, `JS<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `JS::TraceEdge()` with a
//!    pointer to the `JSObject` for the reflector. This notifies the GC, which
//!    will add the object to the graph, and will trace that object as well.
//! 5. When the GC finishes tracing, it [`finalizes`](../index.html#destruction)
//!    any reflectors that were not reachable.
//!
//! The `unsafe_no_jsmanaged_fields!()` macro adds an empty implementation of
//! `JSTraceable` to a datatype.

use app_units::Au;
use canvas_traits::{CanvasGradientStop, LinearGradientStyle, RadialGradientStyle};
use canvas_traits::{CompositionOrBlending, LineCapStyle, LineJoinStyle, RepetitionStyle};
use cssparser::RGBA;
use devtools_traits::{CSSError, TimelineMarkerType, WorkerId};
use dom::abstractworker::SharedRt;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{DomObject, Reflector};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::utils::WindowProxyHandler;
use dom::document::PendingRestyle;
use encoding::types::EncodingRef;
use euclid::{Matrix2D, Matrix4D, Point2D};
use euclid::length::Length as EuclidLength;
use euclid::rect::Rect;
use euclid::size::Size2D;
use html5ever::{Prefix, LocalName, Namespace, QualName};
use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::IncompleteUtf8;
use hyper::header::Headers;
use hyper::method::Method;
use hyper::mime::Mime;
use hyper::status::StatusCode;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use js::glue::{CallObjectTracer, CallValueTracer};
use js::jsapi::{GCTraceKindToAscii, Heap, JSObject, JSTracer, TraceKind};
use js::jsval::JSVal;
use js::rust::Runtime;
use msg::constellation_msg::{BrowsingContextId, FrameType, PipelineId, TopLevelBrowsingContextId};
use net_traits::{Metadata, NetworkError, ReferrerPolicy, ResourceThreads};
use net_traits::filemanager_thread::RelativePos;
use net_traits::image::base::{Image, ImageMetadata};
use net_traits::image_cache::{ImageCache, PendingImageId};
use net_traits::request::{Request, RequestInit};
use net_traits::response::{Response, ResponseBody};
use net_traits::response::HttpsState;
use net_traits::storage_thread::StorageType;
use offscreen_gl_context::GLLimits;
use parking_lot::RwLock;
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
use script_layout_interface::OpaqueStyleAndLayoutData;
use script_layout_interface::reporter::CSSErrorReporter;
use script_layout_interface::rpc::LayoutRPC;
use script_traits::{DocumentActivity, TimerEventId, TimerSource, TouchpadPressurePhase};
use script_traits::{UntrustedNodeAddress, WindowSizeData, WindowSizeType};
use selectors::matching::ElementSelectorFlags;
use serde::{Deserialize, Serialize};
use servo_atoms::Atom;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use smallvec::SmallVec;
use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::hash::{BuildHasher, Hash};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{SystemTime, Instant};
use style::attr::{AttrIdentifier, AttrValue, LengthOrPercentageOrAuto};
use style::context::QuirksMode;
use style::element_state::*;
use style::keyframes::Keyframe;
use style::media_queries::MediaList;
use style::properties::PropertyDeclarationBlock;
use style::selector_parser::{PseudoElement, Snapshot};
use style::shared_lock::{SharedRwLock as StyleSharedRwLock, Locked as StyleLocked};
use style::stylearc::Arc as StyleArc;
use style::stylesheets::{CssRules, FontFaceRule, KeyframesRule, MediaRule};
use style::stylesheets::{NamespaceRule, StyleRule, ImportRule, SupportsRule};
use style::values::specified::Length;
use style::viewport::ViewportRule;
use time::Duration;
use uuid::Uuid;
use webrender_traits::{WebGLBufferId, WebGLError, WebGLFramebufferId, WebGLProgramId};
use webrender_traits::{WebGLRenderbufferId, WebGLShaderId, WebGLTextureId, WebGLVertexArrayId};
use webvr_traits::WebVRGamepadHand;

/// A trait to allow tracing (only) DOM objects.
pub unsafe trait JSTraceable {
    /// Trace `self`.
    unsafe fn trace(&self, trc: *mut JSTracer);
}

unsafe_no_jsmanaged_fields!(CSSError);

unsafe_no_jsmanaged_fields!(EncodingRef);

unsafe_no_jsmanaged_fields!(Reflector);

unsafe_no_jsmanaged_fields!(Duration);

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: &Heap<JSVal>) {
    unsafe {
        if !val.get().is_markable() {
            return;
        }

        trace!("tracing value {}", description);
        CallValueTracer(tracer,
                        val.ptr.get() as *mut _,
                        GCTraceKindToAscii(val.get().trace_kind()));
    }
}

/// Trace the `JSObject` held by `reflector`.
#[allow(unrooted_must_root)]
pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    trace!("tracing reflector {}", description);
    trace_object(tracer, description, reflector.rootable())
}

/// Trace a `JSObject`.
pub fn trace_object(tracer: *mut JSTracer, description: &str, obj: &Heap<*mut JSObject>) {
    unsafe {
        trace!("tracing {}", description);
        CallObjectTracer(tracer,
                         obj.ptr.get() as *mut _,
                         GCTraceKindToAscii(TraceKind::Object));
    }
}

unsafe impl<T: JSTraceable> JSTraceable for Rc<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

unsafe impl<T: JSTraceable> JSTraceable for Arc<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

unsafe impl<T: JSTraceable> JSTraceable for StyleArc<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

unsafe impl<T: JSTraceable + ?Sized> JSTraceable for Box<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

unsafe impl<T: JSTraceable + Copy> JSTraceable for Cell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.get().trace(trc)
    }
}

unsafe impl<T: JSTraceable> JSTraceable for UnsafeCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (*self.get()).trace(trc)
    }
}

unsafe impl<T: JSTraceable> JSTraceable for DOMRefCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow_for_gc_trace().trace(trc)
    }
}

unsafe impl JSTraceable for Heap<*mut JSObject> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if self.get().is_null() {
            return;
        }
        trace_object(trc, "heap object", self);
    }
}

unsafe impl JSTraceable for Heap<JSVal> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        trace_jsval(trc, "heap value", self);
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an unsafe_no_jsmanaged_fields type)
unsafe impl<T: JSTraceable> JSTraceable for Vec<T> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in &*self {
            e.trace(trc);
        }
    }
}

unsafe impl<T: JSTraceable> JSTraceable for VecDeque<T> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in &*self {
            e.trace(trc);
        }
    }
}

unsafe impl<T: JSTraceable> JSTraceable for (T, T, T, T) {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.0.trace(trc);
        self.1.trace(trc);
        self.2.trace(trc);
        self.3.trace(trc);
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an unsafe_no_jsmanaged_fields type)
unsafe impl<T: JSTraceable + 'static> JSTraceable for SmallVec<[T; 1]> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

unsafe impl<T: JSTraceable> JSTraceable for Option<T> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.as_ref().map(|e| e.trace(trc));
    }
}

unsafe impl<T: JSTraceable, U: JSTraceable> JSTraceable for Result<T, U> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        match *self {
            Ok(ref inner) => inner.trace(trc),
            Err(ref inner) => inner.trace(trc),
        }
    }
}

unsafe impl<K, V, S> JSTraceable for HashMap<K, V, S>
    where K: Hash + Eq + JSTraceable,
          V: JSTraceable,
          S: BuildHasher
{
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in &*self {
            k.trace(trc);
            v.trace(trc);
        }
    }
}

unsafe impl<T, S> JSTraceable for HashSet<T, S>
    where T: Hash + Eq + JSTraceable,
          S: BuildHasher
{
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for v in &*self {
            v.trace(trc);
        }
    }
}

unsafe impl<K: Ord + JSTraceable, V: JSTraceable> JSTraceable for BTreeMap<K, V> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in self {
            k.trace(trc);
            v.trace(trc);
        }
    }
}

unsafe impl<A: JSTraceable, B: JSTraceable> JSTraceable for (A, B) {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        let (ref a, ref b) = *self;
        a.trace(trc);
        b.trace(trc);
    }
}

unsafe impl<A: JSTraceable, B: JSTraceable, C: JSTraceable> JSTraceable for (A, B, C) {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        let (ref a, ref b, ref c) = *self;
        a.trace(trc);
        b.trace(trc);
        c.trace(trc);
    }
}

unsafe_no_jsmanaged_fields!(bool, f32, f64, String, AtomicBool, AtomicUsize, Uuid, char);
unsafe_no_jsmanaged_fields!(usize, u8, u16, u32, u64);
unsafe_no_jsmanaged_fields!(isize, i8, i16, i32, i64);
unsafe_no_jsmanaged_fields!(ServoUrl, ImmutableOrigin, MutableOrigin);
unsafe_no_jsmanaged_fields!(Image, ImageMetadata, ImageCache, PendingImageId);
unsafe_no_jsmanaged_fields!(Metadata);
unsafe_no_jsmanaged_fields!(NetworkError);
unsafe_no_jsmanaged_fields!(Atom, Prefix, LocalName, Namespace, QualName);
unsafe_no_jsmanaged_fields!(TrustedPromise);
unsafe_no_jsmanaged_fields!(PropertyDeclarationBlock);
// These three are interdependent, if you plan to put jsmanaged data
// in one of these make sure it is propagated properly to containing structs
unsafe_no_jsmanaged_fields!(DocumentActivity, WindowSizeData, WindowSizeType);
unsafe_no_jsmanaged_fields!(BrowsingContextId, FrameType, PipelineId, TopLevelBrowsingContextId);
unsafe_no_jsmanaged_fields!(TimerEventId, TimerSource);
unsafe_no_jsmanaged_fields!(TimelineMarkerType);
unsafe_no_jsmanaged_fields!(WorkerId);
unsafe_no_jsmanaged_fields!(BufferQueue, QuirksMode, IncompleteUtf8);
unsafe_no_jsmanaged_fields!(Runtime);
unsafe_no_jsmanaged_fields!(Headers, Method);
unsafe_no_jsmanaged_fields!(WindowProxyHandler);
unsafe_no_jsmanaged_fields!(UntrustedNodeAddress);
unsafe_no_jsmanaged_fields!(LengthOrPercentageOrAuto);
unsafe_no_jsmanaged_fields!(RGBA);
unsafe_no_jsmanaged_fields!(StorageType);
unsafe_no_jsmanaged_fields!(CanvasGradientStop, LinearGradientStyle, RadialGradientStyle);
unsafe_no_jsmanaged_fields!(LineCapStyle, LineJoinStyle, CompositionOrBlending);
unsafe_no_jsmanaged_fields!(RepetitionStyle);
unsafe_no_jsmanaged_fields!(WebGLError, GLLimits);
unsafe_no_jsmanaged_fields!(TimeProfilerChan);
unsafe_no_jsmanaged_fields!(MemProfilerChan);
unsafe_no_jsmanaged_fields!(PseudoElement);
unsafe_no_jsmanaged_fields!(Length);
unsafe_no_jsmanaged_fields!(ElementSelectorFlags);
unsafe_no_jsmanaged_fields!(ElementState);
unsafe_no_jsmanaged_fields!(DOMString);
unsafe_no_jsmanaged_fields!(Mime);
unsafe_no_jsmanaged_fields!(AttrIdentifier);
unsafe_no_jsmanaged_fields!(AttrValue);
unsafe_no_jsmanaged_fields!(Snapshot);
unsafe_no_jsmanaged_fields!(PendingRestyle);
unsafe_no_jsmanaged_fields!(HttpsState);
unsafe_no_jsmanaged_fields!(Request);
unsafe_no_jsmanaged_fields!(RequestInit);
unsafe_no_jsmanaged_fields!(SharedRt);
unsafe_no_jsmanaged_fields!(StyleSharedRwLock);
unsafe_no_jsmanaged_fields!(TouchpadPressurePhase);
unsafe_no_jsmanaged_fields!(USVString);
unsafe_no_jsmanaged_fields!(ReferrerPolicy);
unsafe_no_jsmanaged_fields!(Response);
unsafe_no_jsmanaged_fields!(ResponseBody);
unsafe_no_jsmanaged_fields!(ResourceThreads);
unsafe_no_jsmanaged_fields!(StatusCode);
unsafe_no_jsmanaged_fields!(SystemTime);
unsafe_no_jsmanaged_fields!(Instant);
unsafe_no_jsmanaged_fields!(RelativePos);
unsafe_no_jsmanaged_fields!(OpaqueStyleAndLayoutData);
unsafe_no_jsmanaged_fields!(PathBuf);
unsafe_no_jsmanaged_fields!(CSSErrorReporter);
unsafe_no_jsmanaged_fields!(WebGLBufferId);
unsafe_no_jsmanaged_fields!(WebGLFramebufferId);
unsafe_no_jsmanaged_fields!(WebGLProgramId);
unsafe_no_jsmanaged_fields!(WebGLRenderbufferId);
unsafe_no_jsmanaged_fields!(WebGLShaderId);
unsafe_no_jsmanaged_fields!(WebGLTextureId);
unsafe_no_jsmanaged_fields!(WebGLVertexArrayId);
unsafe_no_jsmanaged_fields!(MediaList);
unsafe_no_jsmanaged_fields!(WebVRGamepadHand);

unsafe impl<'a> JSTraceable for &'a str {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<A, B> JSTraceable for fn(A) -> B {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T> JSTraceable for IpcSender<T> where T: Deserialize + Serialize {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

// Safe thanks to the Send bound.
unsafe impl JSTraceable for Box<LayoutRPC + Send + 'static> {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for () {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T> JSTraceable for IpcReceiver<T> where T: Deserialize + Serialize {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T: DomObject> JSTraceable for Trusted<T> {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T: Send> JSTraceable for Receiver<T> {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T: Send> JSTraceable for Sender<T> {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for Matrix2D<f32> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for Matrix4D<f64> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for Point2D<f32> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T> JSTraceable for EuclidLength<u64, T> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for Rect<Au> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for Rect<f32> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for Size2D<i32> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl JSTraceable for Mutex<Option<SharedRt>> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<FontFaceRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<CssRules> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<Keyframe> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<KeyframesRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<ImportRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<SupportsRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<MediaRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<NamespaceRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<StyleRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<ViewportRule> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<PropertyDeclarationBlock> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for RwLock<SharedRt> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

unsafe impl JSTraceable for StyleLocked<MediaList> {
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing.
    }
}

/// Holds a set of JSTraceables that need to be rooted
struct RootedTraceableSet {
    set: Vec<*const JSTraceable>,
}

thread_local!(
    /// TLV Holds a set of JSTraceables that need to be rooted
    static ROOTED_TRACEABLES: RefCell<RootedTraceableSet> =
        RefCell::new(RootedTraceableSet::new());
);

impl RootedTraceableSet {
    fn new() -> RootedTraceableSet {
        RootedTraceableSet {
            set: vec![],
        }
    }

    unsafe fn remove(traceable: *const JSTraceable) {
        ROOTED_TRACEABLES.with(|ref traceables| {
            let mut traceables = traceables.borrow_mut();
            let idx =
                match traceables.set.iter()
                                .rposition(|x| *x == traceable) {
                    Some(idx) => idx,
                    None => unreachable!(),
                };
            traceables.set.remove(idx);
        });
    }

    unsafe fn add(traceable: *const JSTraceable) {
        ROOTED_TRACEABLES.with(|ref traceables| {
            traceables.borrow_mut().set.push(traceable);
        })
    }

    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for traceable in &self.set {
            (**traceable).trace(tracer);
        }
    }
}

/// Roots any JSTraceable thing
///
/// If you have a valid DomObject, use Root.
/// If you have GC things like *mut JSObject or JSVal, use rooted!.
/// If you have an arbitrary number of DomObjects to root, use rooted_vec!.
/// If you know what you're doing, use this.
#[derive(JSTraceable)]
pub struct RootedTraceable<'a, T: 'static + JSTraceable> {
    ptr: &'a T,
}

impl<'a, T: JSTraceable + 'static> RootedTraceable<'a, T> {
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

impl<'a, T: JSTraceable + 'static> Drop for RootedTraceable<'a, T> {
    fn drop(&mut self) {
        unsafe {
            RootedTraceableSet::remove(self.ptr);
        }
    }
}

/// Roots any JSTraceable thing
///
/// If you have a valid DomObject, use Root.
/// If you have GC things like *mut JSObject or JSVal, use rooted!.
/// If you have an arbitrary number of DomObjects to root, use rooted_vec!.
/// If you know what you're doing, use this.
pub struct RootedTraceableBox<T: 'static + JSTraceable> {
    ptr: *mut T,
}

unsafe impl<T: JSTraceable + 'static> JSTraceable for RootedTraceableBox<T> {
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (*self.ptr).trace(tracer);
    }
}

impl<T: JSTraceable + 'static> RootedTraceableBox<T> {
    /// Root a JSTraceable thing for the life of this RootedTraceable
    pub fn new(traceable: T) -> RootedTraceableBox<T> {
        let traceable = Box::into_raw(box traceable);
        unsafe {
            RootedTraceableSet::add(traceable);
        }
        RootedTraceableBox {
            ptr: traceable,
        }
    }
}

impl<T: JSTraceable> Deref for RootedTraceableBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe {
            &*self.ptr
        }
    }
}

impl<T: JSTraceable> DerefMut for RootedTraceableBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.ptr
        }
    }
}

impl<T: JSTraceable + 'static> Drop for RootedTraceableBox<T> {
    fn drop(&mut self) {
        unsafe {
            RootedTraceableSet::remove(self.ptr);
            let _ = Box::from_raw(self.ptr);
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
pub struct RootedVec<'a, T: 'static + JSTraceable> {
    root: &'a mut RootableVec<T>,
}

impl<'a, T: 'static + JSTraceable> RootedVec<'a, T> {
    /// Create a vector of items of type T that is rooted for
    /// the lifetime of this struct
    pub fn new(root: &'a mut RootableVec<T>) -> Self {
        unsafe {
            RootedTraceableSet::add(root);
        }
        RootedVec {
            root: root,
        }
    }
}

impl<'a, T: 'static + JSTraceable + DomObject> RootedVec<'a, JS<T>> {
    /// Create a vector of items of type JS<T> that is rooted for
    /// the lifetime of this struct
    pub fn from_iter<I>(root: &'a mut RootableVec<JS<T>>, iter: I) -> Self
        where I: Iterator<Item = Root<T>>
    {
        unsafe {
            RootedTraceableSet::add(root);
        }
        root.v.extend(iter.map(|item| JS::from_ref(&*item)));
        RootedVec {
            root: root,
        }
    }
}

impl<'a, T: JSTraceable + 'static> Drop for RootedVec<'a, T> {
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
    trace!("tracing stack-rooted traceables");
    ROOTED_TRACEABLES.with(|ref traceables| {
        let traceables = traceables.borrow();
        traceables.trace(tracer);
    });
}
