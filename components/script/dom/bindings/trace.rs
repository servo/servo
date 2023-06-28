/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
//!    For example, for fields of type `Dom<T>`, `Dom<T>::trace()` calls
//!    `trace_reflector()`.
//! 4. `trace_reflector()` calls `Dom::TraceEdge()` with a
//!    pointer to the `JSObject` for the reflector. This notifies the GC, which
//!    will add the object to the graph, and will trace that object as well.
//! 5. When the GC finishes tracing, it [`finalizes`](../index.html#destruction)
//!    any reflectors that were not reachable.
//!
//! The `unsafe_no_jsmanaged_fields!()` macro adds an empty implementation of
//! `JSTraceable` to a datatype.

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::{DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::utils::WindowProxyHandler;
use crate::dom::gpubuffer::GPUBufferState;
use crate::dom::gpucanvascontext::WebGPUContextId;
use crate::dom::gpucommandencoder::GPUCommandEncoderState;
use crate::dom::htmlimageelement::SourceSet;
use crate::dom::htmlmediaelement::{HTMLMediaElementFetchContext, MediaFrameRenderer};
use crate::dom::identityhub::Identities;
use crate::script_runtime::{ContextForRequestInterrupt, StreamConsumer};
use crate::script_thread::IncompleteParserContexts;
use crate::task::TaskBox;
use app_units::Au;
use canvas_traits::webgl::{WebGLReceiver, WebGLSender};
use content_security_policy::CspList;
use crossbeam_channel::{Receiver, Sender};
use cssparser::RGBA;
use devtools_traits::{CSSError, TimelineMarkerType, WorkerId};
use embedder_traits::{EventLoopWaker, MediaMetadata};
use encoding_rs::{Decoder, Encoding};
use euclid::default::{Point2D, Rect, Rotation3D, Transform2D};
use euclid::Length as EuclidLength;
use html5ever::buffer_queue::BufferQueue;
use html5ever::{LocalName, Namespace, Prefix, QualName};
use http::header::HeaderMap;
use http::Method;
use http::StatusCode;
use indexmap::IndexMap;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use js::glue::{CallObjectTracer, CallScriptTracer, CallStringTracer, CallValueTracer};
use js::jsapi::{
    GCTraceKindToAscii, Heap, JSObject, JSScript, JSString, JSTracer, JobQueue, TraceKind,
};
use js::jsval::JSVal;
use js::rust::{GCMethods, Handle, Runtime, Stencil};
use js::typedarray::TypedArray;
use js::typedarray::TypedArrayElement;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use media::WindowGLContext;
use metrics::{InteractiveMetrics, InteractiveWindow};
use mime::Mime;
use msg::constellation_msg::{ServiceWorkerId, ServiceWorkerRegistrationId};
use parking_lot::{Mutex as ParkMutex, RwLock};
use profile_traits::mem::ProfilerChan as MemProfilerChan;
use profile_traits::time::ProfilerChan as TimeProfilerChan;
use script_layout_interface::message::PendingRestyle;
use script_layout_interface::rpc::LayoutRPC;
use script_layout_interface::StyleAndOpaqueLayoutData;
use selectors::matching::ElementSelectorFlags;
use serde::{Deserialize, Serialize};
use servo_arc::Arc as ServoArc;
use servo_atoms::Atom;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt::Display;
use std::hash::{BuildHasher, Hash};
use std::mem;
use std::num::NonZeroU64;
use std::ops::{Deref, DerefMut, Range};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime};
use style::author_styles::AuthorStyles;
use style::stylesheet_set::{AuthorStylesheetSet, DocumentStylesheetSet};
use tendril::fmt::UTF8;
use tendril::stream::LossyDecoder;
use tendril::{StrTendril, TendrilSink};
use time::{Duration, Timespec, Tm};
use uuid::Uuid;
use webrender_api::{DocumentId, ExternalImageId, ImageKey};
use webxr_api::{Finger, Hand, Ray, View};

unsafe_no_jsmanaged_fields!(Tm);
unsafe_no_jsmanaged_fields!(JoinHandle<()>);

/// A trait to allow tracing (only) DOM objects.
pub unsafe trait JSTraceable {
    /// Trace `self`.
    unsafe fn trace(&self, trc: *mut JSTracer);
}

/// Wrapper type for nop traceble
///
/// SAFETY: Inner type must not impl JSTraceable
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NoTrace<T>(pub T);

impl<T: Display> Display for NoTrace<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for NoTrace<T> {
    fn from(item: T) -> Self {
        Self(item)
    }
}

#[allow(unsafe_code)]
unsafe impl<T> JSTraceable for NoTrace<T> {
    #[inline]
    unsafe fn trace(&self, _: *mut ::js::jsapi::JSTracer) {}
}

impl<T: MallocSizeOf> MallocSizeOf for NoTrace<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

unsafe_no_jsmanaged_fields!(Box<dyn TaskBox>, Box<dyn EventLoopWaker>);

unsafe_no_jsmanaged_fields!(IncompleteParserContexts);

unsafe_no_jsmanaged_fields!(ServiceWorkerId);
unsafe_no_jsmanaged_fields!(ServiceWorkerRegistrationId);

unsafe_no_jsmanaged_fields!(CSSError);

unsafe_no_jsmanaged_fields!(&'static Encoding);

unsafe_no_jsmanaged_fields!(Decoder);

unsafe_no_jsmanaged_fields!(Reflector);

unsafe_no_jsmanaged_fields!(Duration);

unsafe_no_jsmanaged_fields!(*mut JobQueue);

unsafe_no_jsmanaged_fields!(Cow<'static, str>);

unsafe_no_jsmanaged_fields!(CspList);

/// Trace a `JSScript`.
pub fn trace_script(tracer: *mut JSTracer, description: &str, script: &Heap<*mut JSScript>) {
    unsafe {
        trace!("tracing {}", description);
        CallScriptTracer(
            tracer,
            script.ptr.get() as *mut _,
            GCTraceKindToAscii(TraceKind::Script),
        );
    }
}

/// Trace a `JSVal`.
pub fn trace_jsval(tracer: *mut JSTracer, description: &str, val: &Heap<JSVal>) {
    unsafe {
        if !val.get().is_markable() {
            return;
        }

        trace!("tracing value {}", description);
        CallValueTracer(
            tracer,
            val.ptr.get() as *mut _,
            GCTraceKindToAscii(val.get().trace_kind()),
        );
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
        CallObjectTracer(
            tracer,
            obj.ptr.get() as *mut _,
            GCTraceKindToAscii(TraceKind::Object),
        );
    }
}

/// Trace a `JSString`.
pub fn trace_string(tracer: *mut JSTracer, description: &str, s: &Heap<*mut JSString>) {
    unsafe {
        trace!("tracing {}", description);
        CallStringTracer(
            tracer,
            s.ptr.get() as *mut _,
            GCTraceKindToAscii(TraceKind::String),
        );
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

unsafe impl<T: JSTraceable> JSTraceable for ServoArc<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

unsafe impl<T: JSTraceable> JSTraceable for RwLock<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.read().trace(trc)
    }
}

unsafe impl<T: JSTraceable + ?Sized> JSTraceable for Box<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

unsafe impl<T: JSTraceable> JSTraceable for [T] {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
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

unsafe impl<T: JSTraceable> JSTraceable for DomRefCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow().trace(trc)
    }
}

unsafe impl<T: JSTraceable> JSTraceable for RefCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow().trace(trc)
    }
}

unsafe impl JSTraceable for Heap<*mut JSScript> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if self.get().is_null() {
            return;
        }
        trace_script(trc, "heap script", self);
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

unsafe impl JSTraceable for Heap<*mut JSString> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        if self.get().is_null() {
            return;
        }
        trace_string(trc, "heap string", self);
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

unsafe impl<T: JSTraceable + Eq + Hash> JSTraceable for indexmap::IndexSet<T> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

unsafe impl<A, B, C, D> JSTraceable for (A, B, C, D)
where
    A: JSTraceable,
    B: JSTraceable,
    C: JSTraceable,
    D: JSTraceable,
{
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
where
    K: Hash + Eq + JSTraceable,
    V: JSTraceable,
    S: BuildHasher,
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
where
    T: Hash + Eq + JSTraceable,
    S: BuildHasher,
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

unsafe impl<K, V, S> JSTraceable for IndexMap<K, V, S>
where
    K: Hash + Eq + JSTraceable,
    V: JSTraceable,
    S: BuildHasher,
{
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in &*self {
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
unsafe_no_jsmanaged_fields!(NonZeroU64);
unsafe_no_jsmanaged_fields!(Error);
unsafe_no_jsmanaged_fields!(ServoUrl, ImmutableOrigin, MutableOrigin);
unsafe_no_jsmanaged_fields!(Atom, Prefix, LocalName, Namespace, QualName);
unsafe_no_jsmanaged_fields!(TrustedPromise);

unsafe_no_jsmanaged_fields!(TimelineMarkerType);
unsafe_no_jsmanaged_fields!(WorkerId);
unsafe_no_jsmanaged_fields!(BufferQueue, StrTendril);
unsafe_no_jsmanaged_fields!(Runtime);
unsafe_no_jsmanaged_fields!(ContextForRequestInterrupt);
unsafe_no_jsmanaged_fields!(HeaderMap, Method);
unsafe_no_jsmanaged_fields!(WindowProxyHandler);
unsafe_no_jsmanaged_fields!(RGBA);
unsafe_no_jsmanaged_fields!(TimeProfilerChan);
unsafe_no_jsmanaged_fields!(MemProfilerChan);
unsafe_no_jsmanaged_fields!(ElementSelectorFlags);
unsafe_no_jsmanaged_fields!(DOMString);
unsafe_no_jsmanaged_fields!(Mime);
unsafe_no_jsmanaged_fields!(PendingRestyle);
unsafe_no_jsmanaged_fields!(USVString);
unsafe_no_jsmanaged_fields!(StatusCode);
unsafe_no_jsmanaged_fields!(SystemTime);
unsafe_no_jsmanaged_fields!(Instant);
unsafe_no_jsmanaged_fields!(StyleAndOpaqueLayoutData);
unsafe_no_jsmanaged_fields!(PathBuf);
unsafe_no_jsmanaged_fields!(DocumentId);
unsafe_no_jsmanaged_fields!(ImageKey);
unsafe_no_jsmanaged_fields!(ExternalImageId);
unsafe_no_jsmanaged_fields!(Arc<ParkMutex<Identities>>);
unsafe_no_jsmanaged_fields!(WebGPUContextId);
unsafe_no_jsmanaged_fields!(GPUBufferState);
unsafe_no_jsmanaged_fields!(GPUCommandEncoderState);
unsafe_no_jsmanaged_fields!(Range<u64>);
unsafe_no_jsmanaged_fields!(
    webxr_api::Registry,
    webxr_api::Session,
    webxr_api::Frame,
    webxr_api::LayerId,
    webxr_api::InputSource,
    webxr_api::InputId,
    webxr_api::Joint,
    webxr_api::HitTestId,
    webxr_api::HitTestResult
);
unsafe_no_jsmanaged_fields!(InteractiveMetrics);
unsafe_no_jsmanaged_fields!(InteractiveWindow);
unsafe_no_jsmanaged_fields!(SourceSet);
unsafe_no_jsmanaged_fields!(Mutex<MediaFrameRenderer>);
unsafe_no_jsmanaged_fields!(Timespec);
unsafe_no_jsmanaged_fields!(HTMLMediaElementFetchContext);
unsafe_no_jsmanaged_fields!(Rotation3D<f64>, Transform2D<f32>);
unsafe_no_jsmanaged_fields!(Point2D<f32>, Rect<Au>);
unsafe_no_jsmanaged_fields!(Rect<f32>);
unsafe_no_jsmanaged_fields!(WindowGLContext);
unsafe_no_jsmanaged_fields!(MediaMetadata);
unsafe_no_jsmanaged_fields!(StreamConsumer);
unsafe_no_jsmanaged_fields!(Stencil);

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

unsafe impl<T> JSTraceable for IpcSender<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

// Safe thanks to the Send bound.
unsafe impl JSTraceable for Box<dyn LayoutRPC + Send + 'static> {
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

unsafe impl<T> JSTraceable for IpcReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
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

unsafe impl<T: Send> JSTraceable for WebGLReceiver<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T: Send> JSTraceable for WebGLSender<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<U> JSTraceable for euclid::Vector2D<f32, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T, U> JSTraceable for euclid::Scale<f32, T, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T, U> JSTraceable for euclid::RigidTransform3D<f32, T, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T, U> JSTraceable for euclid::RigidTransform3D<f64, T, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T, U> JSTraceable for euclid::Transform3D<f32, T, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T, U> JSTraceable for euclid::Transform3D<f64, T, U> {
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

unsafe impl<U> JSTraceable for euclid::Size2D<i32, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<U> JSTraceable for euclid::Size2D<f32, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<U> JSTraceable for euclid::Size2D<u32, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<U> JSTraceable for euclid::Rect<i32, U> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<Space> JSTraceable for Ray<Space> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<Eye> JSTraceable for View<Eye> {
    #[inline]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

unsafe impl<T> JSTraceable for TypedArray<T, Box<Heap<*mut JSObject>>>
where
    T: TypedArrayElement,
{
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.underlying_object().trace(trc);
    }
}

unsafe impl<S> JSTraceable for DocumentStylesheetSet<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for (s, _origin) in self.iter() {
            s.trace(tracer)
        }
    }
}

unsafe impl<S> JSTraceable for AuthorStylesheetSet<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for s in self.iter() {
            s.trace(tracer)
        }
    }
}

unsafe impl<S> JSTraceable for AuthorStyles<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.stylesheets.trace(tracer)
    }
}

unsafe impl<Sink> JSTraceable for LossyDecoder<Sink>
where
    Sink: JSTraceable + TendrilSink<UTF8>,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.inner_sink().trace(tracer);
    }
}

unsafe impl<J> JSTraceable for Hand<J>
where
    J: JSTraceable,
{
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        // exhaustive match so we don't miss new fields
        let Hand {
            ref wrist,
            ref thumb_metacarpal,
            ref thumb_phalanx_proximal,
            ref thumb_phalanx_distal,
            ref thumb_phalanx_tip,
            ref index,
            ref middle,
            ref ring,
            ref little,
        } = *self;
        wrist.trace(trc);
        thumb_metacarpal.trace(trc);
        thumb_phalanx_proximal.trace(trc);
        thumb_phalanx_distal.trace(trc);
        thumb_phalanx_tip.trace(trc);
        index.trace(trc);
        middle.trace(trc);
        ring.trace(trc);
        little.trace(trc);
    }
}

unsafe impl<J> JSTraceable for Finger<J>
where
    J: JSTraceable,
{
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        // exhaustive match so we don't miss new fields
        let Finger {
            ref metacarpal,
            ref phalanx_proximal,
            ref phalanx_intermediate,
            ref phalanx_distal,
            ref phalanx_tip,
        } = *self;
        metacarpal.trace(trc);
        phalanx_proximal.trace(trc);
        phalanx_intermediate.trace(trc);
        phalanx_distal.trace(trc);
        phalanx_tip.trace(trc);
    }
}

/// Holds a set of JSTraceables that need to be rooted
struct RootedTraceableSet {
    set: Vec<*const dyn JSTraceable>,
}

thread_local!(
    /// TLV Holds a set of JSTraceables that need to be rooted
    static ROOTED_TRACEABLES: RefCell<RootedTraceableSet> = RefCell::new(RootedTraceableSet::new());
);

impl RootedTraceableSet {
    fn new() -> RootedTraceableSet {
        RootedTraceableSet { set: vec![] }
    }

    unsafe fn remove(traceable: *const dyn JSTraceable) {
        ROOTED_TRACEABLES.with(|ref traceables| {
            let mut traceables = traceables.borrow_mut();
            let idx = match traceables
                .set
                .iter()
                .rposition(|x| *x as *const () == traceable as *const ())
            {
                Some(idx) => idx,
                None => unreachable!(),
            };
            traceables.set.remove(idx);
        });
    }

    unsafe fn add(traceable: *const dyn JSTraceable) {
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
/// If you have a valid DomObject, use DomRoot.
/// If you have GC things like *mut JSObject or JSVal, use rooted!.
/// If you have an arbitrary number of DomObjects to root, use rooted_vec!.
/// If you know what you're doing, use this.
#[unrooted_must_root_lint::allow_unrooted_interior]
pub struct RootedTraceableBox<T: 'static + JSTraceable> {
    ptr: *mut T,
}

unsafe impl<T: JSTraceable + 'static> JSTraceable for RootedTraceableBox<T> {
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        (*self.ptr).trace(tracer);
    }
}

impl<T: JSTraceable + 'static> RootedTraceableBox<T> {
    /// DomRoot a JSTraceable thing for the life of this RootedTraceableBox
    pub fn new(traceable: T) -> RootedTraceableBox<T> {
        Self::from_box(Box::new(traceable))
    }

    /// Consumes a boxed JSTraceable and roots it for the life of this RootedTraceableBox.
    pub fn from_box(boxed_traceable: Box<T>) -> RootedTraceableBox<T> {
        let traceable = Box::into_raw(boxed_traceable);
        unsafe {
            RootedTraceableSet::add(traceable);
        }
        RootedTraceableBox { ptr: traceable }
    }
}

impl<T> RootedTraceableBox<Heap<T>>
where
    Heap<T>: JSTraceable + 'static,
    T: GCMethods + Copy,
{
    pub fn handle(&self) -> Handle<T> {
        unsafe { Handle::from_raw((*self.ptr).handle()) }
    }
}

impl<T: JSTraceable + MallocSizeOf> MallocSizeOf for RootedTraceableBox<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // Briefly resurrect the real Box value so we can rely on the existing calculations.
        // Then immediately forget about it again to avoid dropping the box.
        let inner = unsafe { Box::from_raw(self.ptr) };
        let size = inner.size_of(ops);
        mem::forget(inner);
        size
    }
}

impl<T: JSTraceable + Default> Default for RootedTraceableBox<T> {
    fn default() -> RootedTraceableBox<T> {
        RootedTraceableBox::new(T::default())
    }
}

impl<T: JSTraceable> Deref for RootedTraceableBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: JSTraceable> DerefMut for RootedTraceableBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
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
/// iterator of `DomRoot`s, `rooted_vec!(let v <- iterator);`.
#[allow(unrooted_must_root)]
#[derive(JSTraceable)]
#[unrooted_must_root_lint::allow_unrooted_interior]
pub struct RootableVec<T: JSTraceable> {
    v: Vec<T>,
}

impl<T: JSTraceable> RootableVec<T> {
    /// Create a vector of items of type T that can be rooted later.
    pub fn new_unrooted() -> RootableVec<T> {
        RootableVec { v: vec![] }
    }
}

/// A vector of items that are rooted for the lifetime 'a.
#[unrooted_must_root_lint::allow_unrooted_interior]
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
        RootedVec { root: root }
    }
}

impl<'a, T: 'static + JSTraceable + DomObject> RootedVec<'a, Dom<T>> {
    /// Create a vector of items of type Dom<T> that is rooted for
    /// the lifetime of this struct
    pub fn from_iter<I>(root: &'a mut RootableVec<Dom<T>>, iter: I) -> Self
    where
        I: Iterator<Item = DomRoot<T>>,
    {
        unsafe {
            RootedTraceableSet::add(root);
        }
        root.v.extend(iter.map(|item| Dom::from_ref(&*item)));
        RootedVec { root: root }
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
