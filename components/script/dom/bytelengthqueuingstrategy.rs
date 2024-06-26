/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use super::bindings::codegen::Bindings::FunctionBinding::Function;
use super::bindings::codegen::Bindings::QueuingStrategyBinding::{
    ByteLengthQueuingStrategyMethods, QueuingStrategyInit,
};
use super::bindings::function::FunctionBinding;
use super::bindings::import::module::{DomObject, DomRoot, Fallible, Reflector};
use super::bindings::reflector::reflect_dom_object_with_proto;
use super::countqueuingstrategy::byte_length_queuing_strategy_size;
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

    /// <https://streams.spec.whatwg.org/#blqs-size>
    fn GetSize(&self) -> Fallible<Rc<Function>> {
        let global = self.reflector_.global();
        // Return this's relevant global object's byte length queuing strategy
        // size function.
        if let Some(fun) = global.get_byte_length_queuing_strategy_size() {
            return Ok(fun);
        }

        // Step 1. Let steps be the following steps, given chunk
        // Note: See ByteLengthQueuingStrategySize instead.

        // Step 2. Let F be !CreateBuiltinFunction(steps, 1, "size", « »,
        // globalObject’s relevant Realm).
        let fun = FunctionBinding::new_native(byte_length_queuing_strategy_size, b"size\0", 1, 0);
        // Step 3. Set globalObject’s byte length queuing strategy size function to
        // a Function that represents a reference to F,
        // with callback context equal to globalObject’s relevant settings object.
        global.set_byte_length_queuing_strategy_size(fun.clone());
        Ok(fun)
    }
}
