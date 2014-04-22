/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};

use js::jsapi::{JSObject, JSTracer, JS_CallTracer, JSTRACE_OBJECT};

use std::cast;
use std::cell::RefCell;
use std::libc;
use std::ptr;
use std::ptr::null;
use serialize::{Encodable, Encoder};

// IMPORTANT: We rely on the fact that we never attempt to encode DOM objects using
//            any encoder but JSTracer. Since we derive trace hooks automatically,
//            we are unfortunately required to use generic types everywhere and
//            unsafely cast to the concrete JSTracer we actually require.

fn get_jstracer<'a, S: Encoder>(s: &'a mut S) -> &'a mut JSTracer {
    unsafe {
        cast::transmute(s)
    }
}

impl<T: Reflectable+Encodable<S>, S: Encoder> Encodable<S> for JS<T> {
    fn encode(&self, s: &mut S) {
        trace_reflector(get_jstracer(s), "", self.reflector());
    }
}

impl<S: Encoder> Encodable<S> for Reflector {
    fn encode(&self, _s: &mut S) {
    }
}

pub trait JSTraceable {
    fn trace(&self, trc: *mut JSTracer);
}

pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    trace_object(tracer, description, reflector.get_jsobject())
}

pub fn trace_object(tracer: *mut JSTracer, description: &str, obj: *JSObject) {
    unsafe {
        description.to_c_str().with_ref(|name| {
            (*tracer).debugPrinter = ptr::null();
            (*tracer).debugPrintIndex = -1;
            (*tracer).debugPrintArg = name as *libc::c_void;
            debug!("tracing {:s}", description);
            JS_CallTracer(tracer as *JSTracer, obj, JSTRACE_OBJECT as u32);
        });
    }
}

/// Encapsulates a type that cannot easily have Encodable derived automagically,
/// but also does not need to be made known to the SpiderMonkey garbage collector.
/// Use only with types that are not associated with a JS reflector and do not contain
/// fields of types associated with JS reflectors.
pub struct Untraceable<T> {
    priv inner: T,
}

impl<T> Untraceable<T> {
    pub fn new(val: T) -> Untraceable<T> {
        Untraceable {
            inner: val
        }
    }
}

impl<S: Encoder, T> Encodable<S> for Untraceable<T> {
    fn encode(&self, _s: &mut S) {
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

/// Encapsulates a type that can be traced but is boxed in a type we don't control
/// (such as RefCell). Wrap a field in Traceable and implement the Encodable trait
/// for that new concrete type to achieve magic compiler-derived trace hooks.
pub struct Traceable<T> {
    priv inner: T
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

impl<T> DerefMut<T> for Traceable<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.inner
    }
}

impl<S: Encoder, T: Encodable<S>> Encodable<S> for Traceable<RefCell<T>> {
    fn encode(&self, s: &mut S) {
        self.borrow().encode(s)
    }
}

impl<S: Encoder> Encodable<S> for Traceable<*JSObject> {
    fn encode(&self, s: &mut S) {
        trace_object(get_jstracer(s), "object", **self)
    }
}
