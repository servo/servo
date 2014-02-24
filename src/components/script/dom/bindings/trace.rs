/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, trace_reflector, Reflector};

use js::jsapi::JSTracer;

use std::cast;
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
