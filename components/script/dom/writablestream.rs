/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::ptr::{self};

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::UnderlyingSinkBinding::UnderlyingSink;
use crate::dom::bindings::codegen::Bindings::WritableStreamBinding::WritableStreamMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, DomObject, Reflector,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::writablestreamdefaultwriter::WritableStreamDefaultWriter;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::dom::bindings::conversions::{ConversionBehavior, ConversionResult};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use crate::dom::bindings::import::module::Fallible;

/// <https://streams.spec.whatwg.org/#ws-class>
#[dom_struct]
pub struct WritableStream {
    reflector_: Reflector,
}

impl WritableStreamMethods<crate::DomTypeHolder> for WritableStream {
    /// <https://streams.spec.whatwg.org/#ws-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        underlying_sink: Option<*mut JSObject>,
        strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<WritableStream>> {
        // If underlyingSink is missing, set it to null.
        rooted!(in(*cx) let underlying_sink_obj = underlying_sink.unwrap_or(ptr::null_mut()));
        
        // Let underlyingSinkDict be underlyingSink, 
        // converted to an IDL value of type UnderlyingSink.
        let underlying_sink_dict = if !underlying_sink_obj.is_null() {
            rooted!(in(*cx) let obj_val = ObjectValue(underlying_sink_obj.get()));
            match UnderlyingSink::new(cx, obj_val.handle()) {
                Ok(ConversionResult::Success(val)) => val,
                Ok(ConversionResult::Failure(error)) => return Err(Error::Type(error.to_string())),
                _ => {
                    return Err(Error::JSFailed);
                },
            }
        } else {
            UnderlyingSink::empty()
        };
        
        
        if underlying_source_dict.type_.is_some() {
            // If underlyingSinkDict["type"] exists, throw a RangeError exception.
            return Err(Error::Range("type is set".to_string()));
        } else {
            // Perform ! InitializeWritableStream(this).
            
            // Let sizeAlgorithm be ! ExtractSizeAlgorithm(strategy).
            let size_algorithm = extract_size_algorithm(strategy);
            
            // Let highWaterMark be ? ExtractHighWaterMark(strategy, 1).
            let high_water_mark = extract_high_water_mark(strategy, 1.0)?;

            // Perform ? SetUpWritableStreamDefaultControllerFromUnderlyingSink
        };
        
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-locked>
    fn Locked(&self) -> bool {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-abort>
    fn Abort(&self, cx: SafeJSContext, reason: SafeHandleValue, _can_gc: CanGc) -> Rc<Promise> {
        todo!()
    }

    /// <https://streams.spec.whatwg.org/#ws-close>
    fn Close(&self, _can_gc: CanGc) -> Rc<Promise> {
        todo!()
    }

    fn GetWriter(&self) -> DomRoot<WritableStreamDefaultWriter> {
        todo!()
    }
}
