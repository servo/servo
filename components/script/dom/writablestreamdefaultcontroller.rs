/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::JSObject;
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::WritableStreamDefaultControllerBinding::WritableStreamDefaultControllerMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::{
    reflect_dom_object, reflect_dom_object_with_proto, DomObject, Reflector,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::writablestream::WritableStream;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

/// <https://streams.spec.whatwg.org/#ws-class>
#[dom_struct]
pub struct WritableStreamDefaultController {
    reflector_: Reflector,
}

impl WritableStreamDefaultControllerMethods<crate::DomTypeHolder>
    for WritableStreamDefaultController
{
    fn Error(&self, cx: SafeJSContext, e: SafeHandleValue) {
        todo!()
    }
}
