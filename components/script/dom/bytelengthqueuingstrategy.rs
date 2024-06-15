use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::{HandleObject, HandleValue};

use super::{
    bindings::{
        codegen::Bindings::{
            FunctionBinding::Function,
            QueuingStrategyBinding::{ByteLengthQueuingStrategyMethods, QueuingStrategyInit},
        },
        import::module::{DomRoot, Reflector},
        reflector::{reflect_dom_object, reflect_dom_object_with_proto},
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
    //https://streams.spec.whatwg.org/#blqs-constructor
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
        let count_queuing_strategy = Self::new_inherited(init);
        reflect_dom_object_with_proto(Box::new(count_queuing_strategy), global, proto)
    }
}

impl ByteLengthQueuingStrategyMethods for ByteLengthQueuingStrategy {
    fn HighWaterMark(&self) -> f64 {
        todo!()
    }

    fn Size(&self) -> Rc<Function> {
        todo!()
    }
}
