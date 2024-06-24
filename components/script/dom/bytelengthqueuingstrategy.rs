/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::c_char;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::gc::{HandleValue, MutableHandleValue};
use js::jsapi::{CallArgs, JSContext, JS_GetFunctionObject, JS_NewFunction};
use js::jsval::JSVal;
use js::rust::HandleObject;

use super::bindings::codegen::Bindings::FunctionBinding::Function;
use super::bindings::codegen::Bindings::QueuingStrategyBinding::{
    ByteLengthQueuingStrategyMethods, QueuingStrategyInit,
};
use super::bindings::import::module::{
    get_dictionary_property, DomObject, DomRoot, Fallible, Reflector,
};
use super::bindings::reflector::reflect_dom_object_with_proto;
use super::types::GlobalScope;

#[dom_struct]
pub struct ByteLengthQueuingStrategy {
    reflector_: Reflector,
    high_water_mark: f64,
}

#[allow(non_snake_case)]
impl ByteLengthQueuingStrategy {
    /// <https://streams.spec.whatwg.org/#blqs-constructor>
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        init: &QueuingStrategyInit,
    ) -> DomRoot<Self> {
        Self::new(global, proto, init.highWaterMark)
    }

    pub fn new_inherited(init: f64) -> Self {
        Self {
            reflector_: Reflector::new(),
            high_water_mark: init,
        }
    }

    pub fn new(global: &GlobalScope, proto: Option<HandleObject>, init: f64) -> DomRoot<Self> {
        reflect_dom_object_with_proto(Box::new(Self::new_inherited(init)), global, proto)
    }
}

impl ByteLengthQueuingStrategyMethods for ByteLengthQueuingStrategy {
    /// <https://streams.spec.whatwg.org/#blqs-high-water-mark>
    fn HighWaterMark(&self) -> f64 {
        self.high_water_mark
    }

    #[allow(unsafe_code)]
    /// <https://streams.spec.whatwg.org/#blqs-size>
    fn GetSize(&self) -> Fallible<Rc<Function>> {
        let global = self.reflector_.global();
        let cx = GlobalScope::get_cx();
        // Return this's relevant global object's byte length queuing strategy
        // size function.
        if let Some(fun) = global.get_byte_length_queuing_strategy_size() {
            return Ok(fun);
        }

        // Step 1. Let steps be the following steps, given chunk
        // Note: See ByteLengthQueuingStrategySize instead.

        unsafe {
            // Step 2. Let F be !CreateBuiltinFunction(steps, 1, "size", « »,
            // globalObject’s relevant Realm).
            let raw_fun = JS_NewFunction(
                *cx,
                Some(byte_length_queuing_strategy_size),
                1,
                0,
                b"size\0".as_ptr() as *const c_char,
            );
            assert!(!raw_fun.is_null());

            // Step 3. Set globalObject’s byte length queuing strategy size function to
            // a Function that represents a reference to F,
            // with callback context equal to globalObject’s relevant settings object.
            let fun_obj = JS_GetFunctionObject(raw_fun);
            let fun = Function::new(cx, fun_obj);
            global.set_byte_length_queuing_strategy_size(fun.clone());
            Ok(fun)
        }
    }
}

/// <https://streams.spec.whatwg.org/#byte-length-queuing-strategy-size-function>
#[allow(unsafe_code)]
unsafe extern "C" fn byte_length_queuing_strategy_size(
    cx: *mut JSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    // Step 1.1: Return ? GetV(chunk, "byteLength").
    rooted!(in(cx) let object = HandleValue::from_raw(args.get(0)).to_object());
    get_dictionary_property(
        cx,
        object.handle(),
        "byteLength",
        MutableHandleValue::from_raw(args.rval()),
    )
    .unwrap_or(false)
}
