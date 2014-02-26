/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};

use js::jsapi::{JSTracer, JS_CallTracer, JSTRACE_OBJECT};

use std::cast;
use std::libc;
use std::ptr;
use std::ptr::null;
use extra::serialize::{Encodable, Encoder};

// IMPORTANT: We rely on the fact that we never attempt to encode DOM objects using
//            any encoder but JSTracer. Since we derive trace hooks automatically,
//            we are unfortunately required to use generic types everywhere and
//            unsafely cast to the concrete JSTracer we actually require.

impl<T: Reflectable+Encodable<S>, S: Encoder> Encodable<S> for JS<T> {
    fn encode(&self, s: &mut S) {
        let s: &mut JSTracer = unsafe { cast::transmute(s) };
        trace_reflector(s, "", self.reflector());
    }
}

impl<S: Encoder> Encodable<S> for Reflector {
    fn encode(&self, _s: &mut S) {
    }
}

pub trait Traceable {
    fn trace(&self, trc: *mut JSTracer);
}

pub fn trace_reflector(tracer: *mut JSTracer, description: &str, reflector: &Reflector) {
    unsafe {
        description.to_c_str().with_ref(|name| {
            (*tracer).debugPrinter = ptr::null();
            (*tracer).debugPrintIndex = -1;
            (*tracer).debugPrintArg = name as *libc::c_void;
            debug!("tracing {:s}", description);
            JS_CallTracer(tracer as *JSTracer, reflector.get_jsobject(),
                          JSTRACE_OBJECT as u32);
        });
    }
}
