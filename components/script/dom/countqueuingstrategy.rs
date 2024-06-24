/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ffi::c_char;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{CallArgs, JSContext, JS_GetFunctionObject, JS_NewFunction};
use js::jsval::{Int32Value, JSVal};
use js::rust::HandleObject;

use super::bindings::codegen::Bindings::FunctionBinding::Function;
use super::bindings::codegen::Bindings::QueuingStrategyBinding::{
    CountQueuingStrategyMethods, QueuingStrategy, QueuingStrategyInit, QueuingStrategySize,
};
use super::bindings::import::module::{DomObject, DomRoot, Error, Fallible, Reflector};
use super::bindings::reflector::reflect_dom_object_with_proto;
use super::types::GlobalScope;

#[dom_struct]
pub struct CountQueuingStrategy {
    reflector_: Reflector,
    high_water_mark: f64,
}

#[allow(non_snake_case)]
impl CountQueuingStrategy {
    /// <https://streams.spec.whatwg.org/#cqs-constructor>
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

impl CountQueuingStrategyMethods for CountQueuingStrategy {
    /// <https://streams.spec.whatwg.org/#cqs-high-water-mark>
    fn HighWaterMark(&self) -> f64 {
        self.high_water_mark
    }

    #[allow(unsafe_code)]
    /// <https://streams.spec.whatwg.org/#cqs-size>
    fn GetSize(&self) -> Fallible<Rc<Function>> {
        let global = self.reflector_.global();
        let cx = GlobalScope::get_cx();
        // Return this's relevant global object's count queuing strategy
        // size function.
        if let Some(fun) = global.get_count_queuing_strategy_size() {
            return Ok(fun);
        }

        // Step 1. Let steps be the following steps:
        // Note: See count_queuing_strategy_size instead.

        unsafe {
            // Step 2. Let F be
            // ! CreateBuiltinFunction(steps, 0, "size", « »,
            //                         globalObject’s relevant Realm).
            let raw_fun = JS_NewFunction(
                *cx,
                Some(count_queuing_strategy_size),
                0,
                0,
                b"size\0".as_ptr() as *const c_char,
            );
            assert!(!raw_fun.is_null());

            // Step 3. Set globalObject’s count queuing strategy size function to
            // a Function that represents a reference to F,
            // with callback context equal to globalObject’s relevant settings object.
            let fun_obj = JS_GetFunctionObject(raw_fun);
            let fun = Function::new(cx, fun_obj);
            global.set_count_queuing_strategy_size(fun.clone());
            Ok(fun)
        }
    }
}

/// <https://streams.spec.whatwg.org/#count-queuing-strategy-size-function>
#[allow(unsafe_code)]
unsafe extern "C" fn count_queuing_strategy_size(
    _cx: *mut JSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    // Step 1.1. Return 1.
    args.rval().set(Int32Value(1));
    true
}

/// Extract the high water mark from a QueuingStrategy.
/// If the high water mark is not set, return the default value.
///
/// <https://streams.spec.whatwg.org/#validate-and-normalize-high-water-mark>
pub fn extract_high_water_mark(strategy: &QueuingStrategy, default_hwm: f64) -> Result<f64, Error> {
    if strategy.highWaterMark.is_none() {
        return Ok(default_hwm);
    }

    let high_water_mark = strategy.highWaterMark.unwrap();
    if high_water_mark.is_nan() || high_water_mark < 0.0 {
        return Err(Error::Range(
            "High water mark must be a non-negative number.".to_string(),
        ));
    }

    Ok(high_water_mark)
}

/// Extract the size algorithm from a QueuingStrategy.
/// If the size algorithm is not set, return a fallback function which always returns 1.
///
/// <https://streams.spec.whatwg.org/#make-size-algorithm-from-size-function>
pub fn extract_size_algorithm(strategy: &QueuingStrategy) -> Rc<QueuingStrategySize> {
    if strategy.size.is_none() {
        #[allow(unsafe_code)]
        unsafe extern "C" fn fallback_strategy_size(
            _cx: *mut JSContext,
            argc: u32,
            vp: *mut JSVal,
        ) -> bool {
            let args = CallArgs::from_vp(vp, argc);
            args.rval().set(Int32Value(1));
            true
        }
        #[allow(unsafe_code)]
        unsafe {
            let cx = GlobalScope::get_cx();
            let raw_fun = JS_NewFunction(
                *cx,
                Some(fallback_strategy_size),
                0,
                0,
                b"size\0".as_ptr() as *const c_char,
            );
            assert!(!raw_fun.is_null());
            let fun_obj = JS_GetFunctionObject(raw_fun);
            return QueuingStrategySize::new(cx, fun_obj).clone();
        }
    }
    strategy.size.as_ref().unwrap().clone()
}
