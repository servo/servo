/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::error::throw_type_error;
use js::gc::{HandleValue, MutableHandleValue};
use js::jsapi::{CallArgs, JSContext};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::{
    ByteLengthQueuingStrategyMethods, QueuingStrategyInit,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::utils::get_dictionary_property;
use crate::dom::types::GlobalScope;
use crate::native_fn;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ByteLengthQueuingStrategy {
    reflector_: Reflector,
    high_water_mark: f64,
}

impl ByteLengthQueuingStrategy {
    pub(crate) fn new_inherited(init: f64) -> Self {
        Self {
            reflector_: Reflector::new(),
            high_water_mark: init,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        init: f64,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(Box::new(Self::new_inherited(init)), global, proto, can_gc)
    }
}

impl ByteLengthQueuingStrategyMethods<crate::DomTypeHolder> for ByteLengthQueuingStrategy {
    /// <https://streams.spec.whatwg.org/#blqs-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        init: &QueuingStrategyInit,
    ) -> DomRoot<Self> {
        Self::new(global, proto, init.highWaterMark, can_gc)
    }
    /// <https://streams.spec.whatwg.org/#blqs-high-water-mark>
    fn HighWaterMark(&self) -> f64 {
        self.high_water_mark
    }

    /// <https://streams.spec.whatwg.org/#blqs-size>
    fn GetSize(&self, _can_gc: CanGc) -> Fallible<Rc<Function>> {
        let global = self.global();
        // Return this's relevant global object's byte length queuing strategy
        // size function.
        if let Some(fun) = global.get_byte_length_queuing_strategy_size() {
            return Ok(fun);
        }

        // Step 1. Let steps be the following steps, given chunk
        // Note: See ByteLengthQueuingStrategySize instead.

        // Step 2. Let F be !CreateBuiltinFunction(steps, 1, "size", « »,
        // globalObject’s relevant Realm).
        let fun = native_fn!(byte_length_queuing_strategy_size, c"size", 1, 0);
        // Step 3. Set globalObject’s byte length queuing strategy size function to
        // a Function that represents a reference to F,
        // with callback context equal to globalObject's relevant settings object.
        global.set_byte_length_queuing_strategy_size(fun.clone());
        Ok(fun)
    }
}

/// <https://streams.spec.whatwg.org/#byte-length-queuing-strategy-size-function>
#[expect(unsafe_code)]
pub(crate) unsafe fn byte_length_queuing_strategy_size(
    cx: *mut JSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    let args = unsafe { CallArgs::from_vp(vp, argc) };

    // Step 1. Let steps be the following steps, given chunk:
    // Step 1.1. Return ? GetV(chunk, "byteLength").
    let chunk = unsafe { HandleValue::from_raw(args.get(0)) };

    // https://tc39.es/ecma262/#sec-getv
    // Let O be ? ToObject(V).
    if chunk.is_undefined() || chunk.is_null() {
        unsafe {
            throw_type_error(
                cx,
                c"ByteLengthQueuingStrategy size called with undefined or nulll",
            )
        };
        return false;
    }

    if !chunk.is_object() {
        // Return ? O.[[Get]]("byteLength", V).
        // undefined for primitives without the property.
        args.rval().set(UndefinedValue());
        return true;
    }

    rooted!(in(cx) let object = chunk.to_object());

    // Return ? O.[[Get]](P, V).
    match unsafe {
        get_dictionary_property(
            cx,
            object.handle(),
            c"byteLength",
            MutableHandleValue::from_raw(args.rval()),
            CanGc::note(),
        )
    } {
        Ok(true) => true,
        Ok(false) => {
            args.rval().set(UndefinedValue());
            true
        },
        Err(()) => false,
    }
}
