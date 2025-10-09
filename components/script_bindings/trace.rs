/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::OnceCell;
use std::fmt::Display;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

use crossbeam_channel::Sender;
use html5ever::interface::{Tracer as HtmlTracer, TreeSink};
use html5ever::tokenizer::{TokenSink, Tokenizer};
use html5ever::tree_builder::TreeBuilder;
use indexmap::IndexMap;
use js::gc::{GCMethods, Handle};
use js::glue::CallObjectTracer;
use js::jsapi::{GCTraceKindToAscii, Heap, JSObject, JSTracer, TraceKind};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use parking_lot::RwLock;
use servo_arc::Arc as ServoArc;
use smallvec::SmallVec;
use style::author_styles::AuthorStyles;
use style::stylesheet_set::{AuthorStylesheetSet, DocumentStylesheetSet};
use tendril::TendrilSink;
use tendril::fmt::UTF8;
use tendril::stream::LossyDecoder;
#[cfg(feature = "webxr")]
use webxr_api::{Finger, Hand};
use xml5ever::interface::TreeSink as XmlTreeSink;
use xml5ever::tokenizer::XmlTokenizer;
use xml5ever::tree_builder::{Tracer as XmlTracer, XmlTreeBuilder};

use crate::JSTraceable;
use crate::error::Error;
use crate::reflector::Reflector;
use crate::str::USVString;

/// Trace the `JSObject` held by `reflector`.
///
/// # Safety
/// tracer must point to a valid, non-null JS tracer.
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub unsafe fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    trace!("tracing reflector {}", description);
    unsafe { trace_object(tracer, description, reflector.rootable()) }
}

/// Trace a `JSObject`.
///
/// # Safety
/// tracer must point to a valid, non-null JS tracer.
pub(crate) unsafe fn trace_object(
    tracer: *mut JSTracer,
    description: &str,
    obj: &Heap<*mut JSObject>,
) {
    unsafe {
        trace!("tracing {}", description);
        CallObjectTracer(
            tracer,
            obj.ptr.get() as *mut _,
            GCTraceKindToAscii(TraceKind::Object),
        );
    }
}

/// For use on non-jsmanaged types
/// Use #[derive(JSTraceable)] on JS managed types
macro_rules! unsafe_no_jsmanaged_fields(
    ($($ty:ty),+) => (
        $(
            #[allow(unsafe_code)]
            unsafe impl crate::JSTraceable for $ty {
                #[inline]
                unsafe fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
                    // Do nothing
                }
            }
        )+
    );
);

unsafe_no_jsmanaged_fields!(USVString);
unsafe_no_jsmanaged_fields!(Error);

/// A trait to allow tracing only DOM sub-objects.
///
/// # Safety
///
/// This trait is unsafe; if it is implemented incorrectly, the GC may end up collecting objects
/// that are still reachable.
pub unsafe trait CustomTraceable {
    /// Trace `self`.
    ///
    /// # Safety
    ///
    /// The `JSTracer` argument must point to a valid `JSTracer` in memory. In addition,
    /// implementors of this method must ensure that all active objects are properly traced
    /// or else the garbage collector may end up collecting objects that are still reachable.
    unsafe fn trace(&self, trc: *mut JSTracer);
}

unsafe impl<T: CustomTraceable> CustomTraceable for Box<T> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        unsafe { (**self).trace(trc) };
    }
}

unsafe impl<T: JSTraceable> CustomTraceable for OnceCell<T> {
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        if let Some(value) = self.get() {
            unsafe { value.trace(tracer) }
        }
    }
}

unsafe impl<T> CustomTraceable for Sender<T> {
    unsafe fn trace(&self, _: *mut JSTracer) {}
}

unsafe impl<T: JSTraceable> CustomTraceable for ServoArc<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        unsafe { (**self).trace(trc) }
    }
}

unsafe impl<T: JSTraceable> CustomTraceable for RwLock<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        unsafe { self.read().trace(trc) }
    }
}

unsafe impl<T: JSTraceable + Eq + Hash> CustomTraceable for indexmap::IndexSet<T> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            unsafe { e.trace(trc) };
        }
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an unsafe_no_jsmanaged_fields type)
unsafe impl<T: JSTraceable + 'static> CustomTraceable for SmallVec<[T; 1]> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            unsafe { e.trace(trc) };
        }
    }
}

unsafe impl<K, V, S> CustomTraceable for IndexMap<K, V, S>
where
    K: Hash + Eq + JSTraceable,
    V: JSTraceable,
    S: BuildHasher,
{
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for (k, v) in self {
            unsafe { k.trace(trc) };
            unsafe { v.trace(trc) };
        }
    }
}

unsafe impl<S> CustomTraceable for DocumentStylesheetSet<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for (s, _origin) in self.iter() {
            unsafe { s.trace(tracer) };
        }
    }
}

unsafe impl<S> CustomTraceable for AuthorStylesheetSet<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for s in self.iter() {
            unsafe { s.trace(tracer) };
        }
    }
}

unsafe impl<S> CustomTraceable for AuthorStyles<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        unsafe { self.stylesheets.trace(tracer) };
    }
}

unsafe impl<Sink> CustomTraceable for LossyDecoder<Sink>
where
    Sink: JSTraceable + TendrilSink<UTF8>,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        unsafe { self.inner_sink().trace(tracer) };
    }
}

#[cfg(feature = "webxr")]
unsafe impl<J> CustomTraceable for Hand<J>
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
        unsafe {
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
}

#[cfg(feature = "webxr")]
unsafe impl<J> CustomTraceable for Finger<J>
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
        unsafe {
            metacarpal.trace(trc);
            phalanx_proximal.trace(trc);
            phalanx_intermediate.trace(trc);
            phalanx_distal.trace(trc);
            phalanx_tip.trace(trc);
        }
    }
}

unsafe impl<Handle: JSTraceable + Clone, Sink: TreeSink<Handle = Handle> + JSTraceable>
    CustomTraceable for TreeBuilder<Handle, Sink>
{
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer<Handle>(*mut JSTracer, PhantomData<Handle>);
        let tracer = Tracer::<Handle>(trc, PhantomData);

        impl<Handle: JSTraceable> HtmlTracer for Tracer<Handle> {
            type Handle = Handle;
            #[cfg_attr(crown, allow(crown::unrooted_must_root))]
            fn trace_handle(&self, node: &Handle) {
                unsafe {
                    node.trace(self.0);
                }
            }
        }

        self.trace_handles(&tracer);
        unsafe { self.sink.trace(trc) };
    }
}

#[allow(unsafe_code)]
unsafe impl<Handle: JSTraceable + Clone, Sink: TokenSink<Handle = Handle> + CustomTraceable>
    CustomTraceable for Tokenizer<Sink>
{
    unsafe fn trace(&self, trc: *mut JSTracer) {
        unsafe { self.sink.trace(trc) };
    }
}

#[allow(unsafe_code)]
unsafe impl<Handle: JSTraceable + Clone, Sink: JSTraceable + XmlTreeSink<Handle = Handle>>
    CustomTraceable for XmlTokenizer<XmlTreeBuilder<Handle, Sink>>
{
    unsafe fn trace(&self, trc: *mut JSTracer) {
        struct Tracer<Handle>(*mut JSTracer, PhantomData<Handle>);
        let tracer = Tracer(trc, PhantomData);

        impl<Handle: JSTraceable> XmlTracer for Tracer<Handle> {
            type Handle = Handle;
            #[cfg_attr(crown, allow(crown::unrooted_must_root))]
            fn trace_handle(&self, node: &Handle) {
                unsafe {
                    node.trace(self.0);
                }
            }
        }

        let tree_builder = &self.sink;
        tree_builder.trace_handles(&tracer);
        unsafe { tree_builder.sink.trace(trc) };
    }
}

/// Roots any JSTraceable thing
///
/// If you have a valid DomObject, use DomRoot.
/// If you have GC things like *mut JSObject or JSVal, use rooted!.
/// If you have an arbitrary number of DomObjects to root, use rooted_vec!.
/// If you know what you're doing, use this.
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_interior)]
pub struct RootedTraceableBox<T: JSTraceable + 'static>(js::gc::RootedTraceableBox<T>);

unsafe impl<T: JSTraceable + 'static> JSTraceable for RootedTraceableBox<T> {
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        unsafe { self.0.trace(tracer) };
    }
}

impl<T: JSTraceable + 'static> RootedTraceableBox<T> {
    /// DomRoot a JSTraceable thing for the life of this RootedTraceableBox
    pub fn new(traceable: T) -> RootedTraceableBox<T> {
        Self(js::gc::RootedTraceableBox::new(traceable))
    }

    /// Consumes a boxed JSTraceable and roots it for the life of this RootedTraceableBox.
    pub fn from_box(boxed_traceable: Box<T>) -> RootedTraceableBox<T> {
        Self(js::gc::RootedTraceableBox::from_box(boxed_traceable))
    }
}

impl<T> RootedTraceableBox<Heap<T>>
where
    Heap<T>: JSTraceable + 'static,
    T: GCMethods + Copy,
{
    pub fn handle(&self) -> Handle<'_, T> {
        self.0.handle()
    }
}

impl<T: JSTraceable + MallocSizeOf> MallocSizeOf for RootedTraceableBox<T> {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        // Briefly resurrect the real Box value so we can rely on the existing calculations.
        // Then immediately forget about it again to avoid dropping the box.
        let inner = unsafe { Box::from_raw(self.0.ptr()) };
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
        self.0.deref()
    }
}

impl<T: JSTraceable> DerefMut for RootedTraceableBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}

/// Wrapper type for nop traceble
///
/// SAFETY: Inner type must not impl JSTraceable
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(crown, crown::trace_in_no_trace_lint::must_not_have_traceable)]
pub(crate) struct NoTrace<T>(pub(crate) T);

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
