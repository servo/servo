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

use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::fmt::Display;
use std::hash::{BuildHasher, Hash};

/// A trait to allow tracing (only) DOM objects.
pub(crate) use js::gc::Traceable as JSTraceable;
use js::glue::{CallScriptTracer, CallStringTracer, CallValueTracer};
use js::jsapi::{GCTraceKindToAscii, Heap, JSScript, JSString, JSTracer, TraceKind};
use js::jsval::JSVal;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
pub(crate) use script_bindings::trace::*;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::htmlimageelement::SourceSet;
use crate::dom::htmlmediaelement::HTMLMediaElementFetchContext;
use crate::dom::windowproxy::WindowProxyHandler;
use crate::script_runtime::StreamConsumer;
use crate::script_thread::IncompleteParserContexts;
use crate::task::TaskBox;

unsafe impl<T: CustomTraceable> CustomTraceable for DomRefCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow().trace(trc)
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

/// HashMap wrapper, that has non-jsmanaged keys
///
/// Not all methods are reexposed, but you can access inner type via .0
#[cfg_attr(crown, crown::trace_in_no_trace_lint::must_not_have_traceable(0))]
#[derive(Clone, Debug)]
pub(crate) struct HashMapTracedValues<K, V, S = RandomState>(pub(crate) HashMap<K, V, S>);

impl<K, V, S: Default> Default for HashMapTracedValues<K, V, S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, V> HashMapTracedValues<K, V, RandomState> {
    /// Wrapper for HashMap::new()
    #[inline]
    #[must_use]
    pub(crate) fn new() -> HashMapTracedValues<K, V, RandomState> {
        Self(HashMap::new())
    }
}

impl<K, V, S> HashMapTracedValues<K, V, S> {
    #[inline]
    pub(crate) fn iter(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.0.iter()
    }

    #[inline]
    pub(crate) fn drain(&mut self) -> std::collections::hash_map::Drain<'_, K, V> {
        self.0.drain()
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K, V, S> HashMapTracedValues<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    pub(crate) fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    #[inline]
    pub(crate) fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.get(k)
    }

    #[inline]
    pub(crate) fn get_mut<Q: Hash + Eq + ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.0.get_mut(k)
    }

    #[inline]
    pub(crate) fn contains_key<Q: Hash + Eq + ?Sized>(&self, k: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
    {
        self.0.contains_key(k)
    }

    #[inline]
    pub(crate) fn remove<Q: Hash + Eq + ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.0.remove(k)
    }

    #[inline]
    pub(crate) fn entry(&mut self, key: K) -> std::collections::hash_map::Entry<'_, K, V> {
        self.0.entry(key)
    }
}

impl<K, V, S> MallocSizeOf for HashMapTracedValues<K, V, S>
where
    K: Eq + Hash + MallocSizeOf,
    V: MallocSizeOf,
    S: BuildHasher,
{
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.0.size_of(ops)
    }
}

#[allow(unsafe_code)]
unsafe impl<K, V: JSTraceable, S> JSTraceable for HashMapTracedValues<K, V, S> {
    #[inline]
    unsafe fn trace(&self, trc: *mut ::js::jsapi::JSTracer) {
        for v in self.0.values() {
            v.trace(trc);
        }
    }
}

unsafe_no_jsmanaged_fields!(Box<dyn TaskBox>);

unsafe_no_jsmanaged_fields!(IncompleteParserContexts);

#[allow(dead_code)]
/// Trace a `JSScript`.
pub(crate) fn trace_script(tracer: *mut JSTracer, description: &str, script: &Heap<*mut JSScript>) {
    unsafe {
        trace!("tracing {}", description);
        CallScriptTracer(
            tracer,
            script.ptr.get() as *mut _,
            GCTraceKindToAscii(TraceKind::Script),
        );
    }
}

#[allow(dead_code)]
/// Trace a `JSVal`.
pub(crate) fn trace_jsval(tracer: *mut JSTracer, description: &str, val: &Heap<JSVal>) {
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

#[allow(dead_code)]
/// Trace a `JSString`.
pub(crate) fn trace_string(tracer: *mut JSTracer, description: &str, s: &Heap<*mut JSString>) {
    unsafe {
        trace!("tracing {}", description);
        CallStringTracer(
            tracer,
            s.ptr.get() as *mut _,
            GCTraceKindToAscii(TraceKind::String),
        );
    }
}

unsafe impl<T: JSTraceable> JSTraceable for DomRefCell<T> {
    unsafe fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow().trace(trc)
    }
}

unsafe_no_jsmanaged_fields!(TrustedPromise);

unsafe_no_jsmanaged_fields!(WindowProxyHandler);
unsafe_no_jsmanaged_fields!(SourceSet);
unsafe_no_jsmanaged_fields!(HTMLMediaElementFetchContext);
unsafe_no_jsmanaged_fields!(StreamConsumer);

unsafe impl<T: DomObject> JSTraceable for Trusted<T> {
    #[inline]
    unsafe fn trace(&self, _: *mut JSTracer) {
        // Do nothing
    }
}
