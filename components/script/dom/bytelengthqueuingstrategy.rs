/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::JS_NewFunction;
use js::rust::HandleObject;

use super::{
    bindings::{
        codegen::Bindings::{
            FunctionBinding::Function,
            QueuingStrategyBinding::{ByteLengthQueuingStrategyMethods, QueuingStrategyInit},
        },
        import::module::{DomRoot, Fallible, Reflector},
        reflector::reflect_dom_object_with_proto,
    },
    types::GlobalScope,
};

#[dom_struct]
pub struct ByteLengthQueuingStrategy {
    reflector_: Reflector,
    high_water_mark: f64,
}

#[allow(non_snake_case)]
impl ByteLengthQueuingStrategy {
    // https://streams.spec.whatwg.org/#blqs-constructor
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
    // https://streams.spec.whatwg.org/#blqs-high-water-mark
    fn HighWaterMark(&self) -> f64 {
        self.high_water_mark
    }

    fn GetSize(&self) -> Fallible<Rc<Function>> {
        todo!()
    }
}
