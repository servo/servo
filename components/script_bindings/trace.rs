/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::OnceCell;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;

use crossbeam_channel::Sender;
use html5ever::interface::{Tracer as HtmlTracer, TreeSink};
use html5ever::tokenizer::{TokenSink, Tokenizer};
use html5ever::tree_builder::TreeBuilder;
use indexmap::IndexMap;
use js::glue::CallObjectTracer;
use js::jsapi::{GCTraceKindToAscii, Heap, JSObject, JSTracer, TraceKind};
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
use crate::str::{DOMString, USVString};

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
pub unsafe fn trace_object(tracer: *mut JSTracer, description: &str, obj: &Heap<*mut JSObject>) {
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

unsafe_no_jsmanaged_fields!(DOMString);
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
        (**self).trace(trc);
    }
}

unsafe impl<T: JSTraceable> CustomTraceable for OnceCell<T> {
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        if let Some(value) = self.get() {
            value.trace(tracer)
        }
    }
}

unsafe impl<T> CustomTraceable for Sender<T> {
    unsafe fn trace(&self, _: *mut JSTracer) {}
}

unsafe impl<T: JSTraceable> CustomTraceable for ServoArc<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (**self).trace(trc)
    }
}

unsafe impl<T: JSTraceable> CustomTraceable for RwLock<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.read().trace(trc)
    }
}

unsafe impl<T: JSTraceable + Eq + Hash> CustomTraceable for indexmap::IndexSet<T> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
        }
    }
}

// XXXManishearth Check if the following three are optimized to no-ops
// if e.trace() is a no-op (e.g it is an unsafe_no_jsmanaged_fields type)
unsafe impl<T: JSTraceable + 'static> CustomTraceable for SmallVec<[T; 1]> {
    #[inline]
    unsafe fn trace(&self, trc: *mut JSTracer) {
        for e in self.iter() {
            e.trace(trc);
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
            k.trace(trc);
            v.trace(trc);
        }
    }
}

unsafe impl<S> CustomTraceable for DocumentStylesheetSet<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for (s, _origin) in self.iter() {
            s.trace(tracer)
        }
    }
}

unsafe impl<S> CustomTraceable for AuthorStylesheetSet<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        for s in self.iter() {
            s.trace(tracer)
        }
    }
}

unsafe impl<S> CustomTraceable for AuthorStyles<S>
where
    S: JSTraceable + ::style::stylesheets::StylesheetInDocument + PartialEq + 'static,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.stylesheets.trace(tracer)
    }
}

unsafe impl<Sink> CustomTraceable for LossyDecoder<Sink>
where
    Sink: JSTraceable + TendrilSink<UTF8>,
{
    unsafe fn trace(&self, tracer: *mut JSTracer) {
        self.inner_sink().trace(tracer);
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
        metacarpal.trace(trc);
        phalanx_proximal.trace(trc);
        phalanx_intermediate.trace(trc);
        phalanx_distal.trace(trc);
        phalanx_tip.trace(trc);
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
        self.sink.trace(trc);
    }
}

#[allow(unsafe_code)]
unsafe impl<Handle: JSTraceable + Clone, Sink: TokenSink<Handle = Handle> + CustomTraceable>
    CustomTraceable for Tokenizer<Sink>
{
    unsafe fn trace(&self, trc: *mut JSTracer) {
        self.sink.trace(trc);
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
        tree_builder.sink.trace(trc);
    }
}
