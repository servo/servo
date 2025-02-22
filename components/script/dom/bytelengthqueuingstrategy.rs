/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::gc::{HandleValue, MutableHandleValue};
use js::jsapi::{CallArgs, JSContext};
use js::jsval::JSVal;
use js::rust::HandleObject;

use super::bindings::codegen::Bindings::FunctionBinding::Function;
use super::bindings::codegen::Bindings::QueuingStrategyBinding::{
    ByteLengthQueuingStrategyMethods, QueuingStrategyInit,
};
use super::bindings::error::Fallible;
use super::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use super::bindings::root::DomRoot;
use super::types::GlobalScope;
use crate::dom::bindings::import::module::get_dictionary_property;
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
        // with callback context equal to globalObject’s relevant settings object.
        global.set_byte_length_queuing_strategy_size(fun.clone());
        Ok(fun)
    }
}

/// <https://streams.spec.whatwg.org/#byte-length-queuing-strategy-size-function>
#[allow(unsafe_code)]
pub(crate) unsafe fn byte_length_queuing_strategy_size(
    cx: *mut JSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    let args = CallArgs::from_vp(vp, argc);

    // Step 1.1: Return ? GetV(chunk, "byteLength").
    let val = HandleValue::from_raw(args.get(0));

    // https://tc39.es/ecma262/multipage/abstract-operations.html#sec-getv
    // Let O be ? ToObject(V).
    if !val.is_object() {
        return false;
    }
    rooted!(in(cx) let object = val.to_object());

    // Return ? O.[[Get]](P, V).
    get_dictionary_property(
        cx,
        object.handle(),
        "byteLength",
        MutableHandleValue::from_raw(args.rval()),
    )
    .unwrap_or(false)
}
