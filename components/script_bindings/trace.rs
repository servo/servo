/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use js::glue::CallObjectTracer;
use js::jsapi::{GCTraceKindToAscii, Heap, JSObject, JSTracer, TraceKind};

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
    trace_object(tracer, description, reflector.rootable())
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
