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
//! 2. `_trace` calls `Foo::trace()` (an implementation of `JSTraceable`,
//!    defined in `InheritTypes.rs`).
//! 3. `Foo::trace()` calls `Foo::encode()` (an implementation of `Encodable`).
//!    This implementation is typically derived by a `#[deriving(Encodable)]`
//!    annotation on the Rust struct.
//! 4. For all fields (except those wrapped in `Untraceable`), `Foo::encode()`
//!    calls `encode()` on the field.
//!
//!    For example, for fields of type `JS<T>`, `JS<T>::encode()` calls
//!    `trace_reflector()`.
//! 6. `trace_reflector()` calls `trace_object()` with the `JSObject` for the
//!    reflector.
//! 7. `trace_object()` calls `JS_CallTracer()` to notify the GC, which will
//!    add the object to the graph, and will trace that object as well.

use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};

use js::jsapi::{JSObject, JSTracer, JS_CallTracer, JSTRACE_OBJECT};
use js::jsval::JSVal;

use libc;
use std::mem;
use std::cell::{Cell, RefCell};
use serialize::{Encodable, Encoder};

// IMPORTANT: We rely on the fact that we never attempt to encode DOM objects using
//            any encoder but JSTracer. Since we derive trace hooks automatically,
//            we are unfortunately required to use generic types everywhere and
//            unsafely cast to the concrete JSTracer we actually require.

fn get_jstracer<'a, S: Encoder<E>, E>(s: &'a mut S) -> &'a mut JSTracer {
    unsafe {
        mem::transmute(s)
    }
}

impl<T: Reflectable+Encodable<S, E>, S: Encoder<E>, E> Encodable<S, E> for JS<T> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        trace_reflector(get_jstracer(s), "", self.reflector());
        Ok(())
    }
}

impl<S: Encoder<E>, E> Encodable<S, E> for Reflector {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

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

/// Encapsulates a type that cannot easily have `Encodable` derived automagically,
/// but also does not need to be made known to the SpiderMonkey garbage collector.
///
/// Use only with types that are not associated with a JS reflector and do not contain
/// fields of types associated with JS reflectors.
///
/// This should really only be used for types that are from other crates,
/// so we can't implement `Encodable`. See more details: mozilla#2662.
pub struct Untraceable<T> {
    inner: T,
}

impl<T> Untraceable<T> {
    pub fn new(val: T) -> Untraceable<T> {
        Untraceable {
            inner: val
        }
    }
}

impl<S: Encoder<E>, E, T> Encodable<S, E> for Untraceable<T> {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

impl<T> Deref<T> for Untraceable<T> {
    fn deref<'a>(&'a self) -> &'a T {
        &self.inner
    }
}

impl<T> DerefMut<T> for Untraceable<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.inner
    }
}

/// Encapsulates a type that can be traced but is boxed in a type we don't
/// control (such as `RefCell`).
///
/// Wrap a field in Traceable and implement the `Encodable` trait
/// for that new concrete type to achieve magic compiler-derived trace hooks.
///
/// We always prefer this, in case the contained type ever changes to something that should be traced.
/// See more details: mozilla#2662.
#[deriving(PartialEq, Clone)]
pub struct Traceable<T> {
    inner: T
}

impl<T> Traceable<T> {
    pub fn new(val: T) -> Traceable<T> {
        Traceable {
            inner: val
        }
    }
}

impl<T> Deref<T> for Traceable<T> {
    fn deref<'a>(&'a self) -> &'a T {
        &self.inner
    }
}

impl<S: Encoder<E>, E, T: Encodable<S, E>> Encodable<S, E> for Traceable<RefCell<T>> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        self.borrow().encode(s)
    }
}

impl<S: Encoder<E>, E, T: Encodable<S, E>+Copy> Encodable<S, E> for Traceable<Cell<T>> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        self.deref().get().encode(s)
    }
}

impl<S: Encoder<E>, E> Encodable<S, E> for Traceable<*mut JSObject> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        trace_object(get_jstracer(s), "object", **self);
        Ok(())
    }
}

impl<S: Encoder<E>, E> Encodable<S, E> for Traceable<JSVal> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        trace_jsval(get_jstracer(s), "val", **self);
        Ok(())
    }
}
